import ExpenseLedger
import Foundation

@main
struct ExpenseSummary {
    private static let usage = """
        Expense Ledger — a local expense journal with exact money and monthly budgets

        Usage: ExpenseSummary [--ledger PATH] COMMAND
          add DATE CATEGORY AMOUNT DESCRIPTION [REFERENCE]
          import FILE.json             Import rows atomically; repeated references are safe
          list [YYYY-MM]               List all entries or one month
          budget CATEGORY AMOUNT       Set a recurring monthly category budget
          report YYYY-MM               Spending, budget, and remaining amount per category
          remove REFERENCE             Remove an incorrect entry
          export                       Write a complete ledger backup to stdout
          help

        Amounts are positive decimals, e.g. 12.50. The ledger uses one currency (EUR).
        Default storage: .data/expenses.json. Import rows: reference, date, category,
        description, amount (a decimal string). Dates use YYYY-MM-DD.
        """

    static func main() {
        do { try execute(Array(CommandLine.arguments.dropFirst())) } catch {
            FileHandle.standardError.write(
                Data("Expense Ledger: \(error.localizedDescription)\n".utf8))
            exit(1)
        }
    }

    private static func execute(_ arguments: [String]) throws {
        var args = arguments
        var path = ".data/expenses.json"
        if args.first == "--ledger" {
            guard args.count >= 3 else { throw LedgerError.invalid(usage) }
            path = args[1]
            args.removeFirst(2)
        }
        guard let command = args.first else {
            print(usage)
            return
        }
        args.removeFirst()
        if ["help", "--help", "-h"].contains(command) {
            print(usage)
            return
        }
        let file = LedgerFile(url: URL(fileURLWithPath: path))
        let output = try file.transaction { ledger -> String in
            switch command {
            case "add" where (4...5).contains(args.count):
                let reference = args.count == 5 ? args[4] : UUID().uuidString.lowercased()
                let added = try ledger.importRows([
                    ImportRow(
                        reference: reference, date: args[0], category: args[1],
                        description: args[3], amount: args[2])
                ])
                return "\(added == 1 ? "Added" : "Already recorded") \(reference)"
            case "import" where args.count == 1:
                let rows = try JSONDecoder().decode(
                    [ImportRow].self, from: Data(contentsOf: URL(fileURLWithPath: args[0])))
                return "Imported \(try ledger.importRows(rows)) of \(rows.count) expenses."
            case "budget" where args.count == 2:
                try ledger.setBudget(category: args[0], amount: args[1])
                return "Monthly budget saved."
            case "remove" where args.count == 1:
                try ledger.remove(reference: args[0])
                return "Removed \(args[0])."
            case "list" where args.count <= 1:
                let entries = try ledger.filtered(month: args.first)
                return
                    (["Date\tCategory\tAmount (\(ledger.currency))\tReference\tDescription"]
                    + entries.map {
                        "\($0.date)\t\($0.category)\t\(Money.format($0.cents))\t\($0.reference)\t\($0.description)"
                    } + ["Total: \(Money.format(try Ledger.total(entries))) \(ledger.currency)"])
                    .joined(separator: "\n")
            case "report" where args.count == 1:
                let rows = try ledger.report(month: args[0])
                return
                    (["\(args[0]) · \(ledger.currency)", "Category\tSpent\tBudget\tRemaining"]
                    + rows.map {
                        "\($0.category)\t\(Money.format($0.spent))\t\($0.budget.map(Money.format) ?? "—")\t\($0.remaining.map(Money.format) ?? "—")"
                    } + [
                        "Total: \(Money.format(try Ledger.total(ledger.filtered(month: args[0])))) \(ledger.currency)"
                    ]).joined(separator: "\n")
            case "export" where args.isEmpty:
                return String(decoding: try file.encoded(ledger), as: UTF8.self)
            default: throw LedgerError.invalid(usage)
            }
        }
        print(output)
    }
}
