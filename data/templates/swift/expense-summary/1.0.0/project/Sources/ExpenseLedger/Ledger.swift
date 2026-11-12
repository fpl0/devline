import Foundation

public enum LedgerError: LocalizedError {
    case invalid(String)
    public var errorDescription: String? {
        switch self {
        case .invalid(let message): message
        }
    }
}

/// Money is always represented in minor units, never binary floating point.
public enum Money {
    public static func parse(_ value: String) throws -> Int64 {
        let parts = value.split(separator: ".", omittingEmptySubsequences: false)
        guard (1...2).contains(parts.count), !parts[0].isEmpty,
            parts.allSatisfy({ $0.allSatisfy { $0.isASCII && $0.isNumber } }),
            parts.count == 1 || (1...2).contains(parts[1].count),
            let whole = Int64(parts[0])
        else {
            throw LedgerError.invalid(
                "Invalid amount '\(value)'; use a positive decimal with at most two fractional digits."
            )
        }
        let fraction = parts.count == 1 ? 0 : Int64(parts[1])! * (parts[1].count == 1 ? 10 : 1)
        let (base, overflow) = whole.multipliedReportingOverflow(by: 100)
        let (cents, sumOverflow) = base.addingReportingOverflow(fraction)
        guard !overflow, !sumOverflow, cents > 0 else {
            throw LedgerError.invalid("Amount is zero or too large.")
        }
        return cents
    }
    public static func format(_ cents: Int64) -> String {
        let absolute = cents.magnitude
        return
            "\(cents < 0 ? "-" : "")\(absolute / 100).\(String(format: "%02llu", absolute % 100))"
    }
}

public struct ImportRow: Codable, Equatable, Sendable {
    public let reference: String
    public let date: String
    public let category: String
    public let description: String
    public let amount: String
    public init(
        reference: String, date: String, category: String, description: String, amount: String
    ) {
        self.reference = reference
        self.date = date
        self.category = category
        self.description = description
        self.amount = amount
    }
}

public struct Entry: Codable, Equatable, Identifiable, Sendable {
    public var id: String { reference }
    public let reference: String
    public let date: String
    public let category: String
    public let description: String
    public let cents: Int64
}

public struct CategoryReport: Equatable, Sendable {
    public let category: String
    public let spent: Int64
    public let budget: Int64?
    public var remaining: Int64? { budget.map { $0 - spent } }
}

public struct Ledger: Codable, Equatable, Sendable {
    public private(set) var schemaVersion = 1
    public private(set) var currency: String
    public private(set) var entries: [Entry] = []
    public private(set) var monthlyBudgets: [String: Int64] = [:]
    public init(currency: String = "EUR") throws {
        guard currency.count == 3, currency.utf8.allSatisfy({ (65...90).contains($0) }) else {
            throw LedgerError.invalid(
                "Currency must be a three-letter uppercase code, such as EUR.")
        }
        self.currency = currency
    }

    public static func validateDate(_ date: String, monthOnly: Bool = false) throws {
        let pattern = monthOnly ? #"^[0-9]{4}-[0-9]{2}$"# : #"^[0-9]{4}-[0-9]{2}-[0-9]{2}$"#
        guard date.range(of: pattern, options: .regularExpression) != nil else {
            throw LedgerError.invalid("Use \(monthOnly ? "YYYY-MM" : "YYYY-MM-DD") dates.")
        }
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.dateFormat = monthOnly ? "yyyy-MM" : "yyyy-MM-dd"
        formatter.isLenient = false
        guard let parsed = formatter.date(from: date), formatter.string(from: parsed) == date else {
            throw LedgerError.invalid("Invalid calendar date: \(date).")
        }
    }
    private static func text(_ value: String, field: String, limit: Int) throws -> String {
        let clean = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !clean.isEmpty, clean.count <= limit,
            clean.unicodeScalars.allSatisfy({ !CharacterSet.controlCharacters.contains($0) })
        else {
            throw LedgerError.invalid(
                "\(field) must contain 1–\(limit) characters without control characters.")
        }
        return clean
    }

    /// Validate the whole batch before publishing it. Exact duplicate references are idempotent.
    @discardableResult
    public mutating func importRows(_ rows: [ImportRow]) throws -> Int {
        var next = entries
        var known = Dictionary(uniqueKeysWithValues: entries.map { ($0.reference, $0) })
        var added = 0
        for row in rows {
            try Self.validateDate(row.date)
            let entry = Entry(
                reference: try Self.text(row.reference, field: "Reference", limit: 100),
                date: row.date,
                category: try Self.text(row.category, field: "Category", limit: 60),
                description: try Self.text(row.description, field: "Description", limit: 500),
                cents: try Money.parse(row.amount)
            )
            if let existing = known[entry.reference] {
                guard existing == entry else {
                    throw LedgerError.invalid(
                        "Reference '\(entry.reference)' already has different data.")
                }
            } else {
                known[entry.reference] = entry
                next.append(entry)
                added += 1
            }
        }
        _ = try Self.total(next)
        entries = next.sorted { ($0.date, $0.reference) < ($1.date, $1.reference) }
        return added
    }
    public mutating func setBudget(category: String, amount: String) throws {
        monthlyBudgets[try Self.text(category, field: "Category", limit: 60)] = try Money.parse(
            amount)
    }
    public mutating func remove(reference: String) throws {
        guard let index = entries.firstIndex(where: { $0.reference == reference }) else {
            throw LedgerError.invalid("Unknown reference '\(reference)'.")
        }
        entries.remove(at: index)
    }
    public func filtered(month: String?) throws -> [Entry] {
        if let month { try Self.validateDate(month, monthOnly: true) }
        return entries.filter { month == nil || $0.date.hasPrefix(month! + "-") }
    }
    public func report(month: String) throws -> [CategoryReport] {
        let groups = Dictionary(grouping: try filtered(month: month), by: \.category)
        return try Set(groups.keys).union(monthlyBudgets.keys).sorted().map {
            CategoryReport(
                category: $0, spent: try Self.total(groups[$0] ?? []), budget: monthlyBudgets[$0])
        }
    }
    public static func total(_ entries: [Entry]) throws -> Int64 {
        try entries.reduce(Int64(0)) { total, entry in
            let (result, overflow) = total.addingReportingOverflow(entry.cents)
            guard !overflow else {
                throw LedgerError.invalid("The ledger total exceeds supported precision.")
            }
            return result
        }
    }
    public func validated() throws -> Ledger {
        guard schemaVersion == 1 else {
            throw LedgerError.invalid("Unsupported ledger schema; the file has not been changed.")
        }
        var checked = try Ledger(currency: currency)
        try checked.importRows(
            entries.map {
                ImportRow(
                    reference: $0.reference, date: $0.date, category: $0.category,
                    description: $0.description, amount: Money.format($0.cents))
            })
        guard checked.entries.count == entries.count else {
            throw LedgerError.invalid("Duplicate references in ledger.")
        }
        for (category, cents) in monthlyBudgets {
            try checked.setBudget(category: category, amount: Money.format(cents))
        }
        return checked
    }
}
