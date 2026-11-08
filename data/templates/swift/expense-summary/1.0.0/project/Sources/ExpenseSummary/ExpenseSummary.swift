import Foundation

private struct Expense: Decodable {
    let category: String
    let cents: Int64
}

private enum InputError: LocalizedError {
    case usage, invalidExpense, totalOverflow

    var errorDescription: String? {
        switch self {
        case .usage: "Usage: ExpenseSummary expenses.json"
        case .invalidExpense: "Expenses need a category without line breaks and an amount greater than zero."
        case .totalOverflow: "The expenses exceed the maximum supported total."
        }
    }
}

@main
struct ExpenseSummary {
    static func main() {
        do {
            try summarize()
        } catch {
            FileHandle.standardError.write(Data("\(error.localizedDescription)\n".utf8))
            exit(1)
        }
    }

    private static func summarize() throws {
        guard CommandLine.arguments.count == 2 else { throw InputError.usage }
        let file = URL(fileURLWithPath: CommandLine.arguments[1])
        let expenses = try JSONDecoder().decode([Expense].self, from: Data(contentsOf: file))
        var totals: [String: Int64] = [:]
        var total: Int64 = 0
        for expense in expenses {
            let category = expense.category.trimmingCharacters(in: .whitespaces)
            guard !category.isEmpty, category.rangeOfCharacter(from: .newlines) == nil,
                  expense.cents > 0 else { throw InputError.invalidExpense }
            let (next, overflow) = total.addingReportingOverflow(expense.cents)
            guard !overflow else { throw InputError.totalOverflow }
            total = next
            totals[category, default: 0] += expense.cents
        }
        for (category, cents) in totals.sorted(by: { $0.key < $1.key }) {
            print("\(category): \(amount(cents))")
        }
        print("Total: \(amount(total))")
    }

    private static func amount(_ cents: Int64) -> String {
        let remainder = cents % 100
        return "\(cents / 100).\(remainder < 10 ? "0" : "")\(remainder)"
    }
}
