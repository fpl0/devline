import ExpenseLedger
import Foundation
import Testing

private func row(
    _ reference: String = "receipt-1", date: String = "2026-11-12", category: String = "Food",
    amount: String = "12.50"
) -> ImportRow {
    ImportRow(
        reference: reference, date: date, category: category, description: "Lunch", amount: amount)
}

@Test func moneyIsExactAndBounded() throws {
    #expect(try Money.parse("0.01") == 1)
    #expect(try Money.parse("12.5") == 1250)
    #expect(try Money.parse("92233720368547758.07") == Int64.max)
    #expect(Money.format(-123) == "-1.23")
    for input in [
        "", "-1", "0", "1.001", "1e2", "1.", ".5", "+3", "NaN", "９", "92233720368547758.08",
    ] {
        #expect(throws: LedgerError.self) { try Money.parse(input) }
    }
}

@Test func importingIsIdempotentAndConflictsAreAtomic() throws {
    var ledger = try Ledger()
    #expect(try ledger.importRows([row(), row()]) == 1)
    let before = ledger
    #expect(try ledger.importRows([row()]) == 0)
    #expect(throws: LedgerError.self) { try ledger.importRows([row("new"), row(amount: "13")]) }
    #expect(ledger == before)
    #expect(throws: LedgerError.self) {
        try ledger.importRows([row("valid"), row("bad", date: "2026-02-30")])
    }
    #expect(ledger == before)
}

@Test func monthlyBudgetIncludesUnspentCategoriesAndOverspend() throws {
    var ledger = try Ledger()
    try ledger.importRows([row(), row("old", date: "2026-10-01", amount: "100")])
    try ledger.setBudget(category: "Food", amount: "10")
    try ledger.setBudget(category: "Travel", amount: "50")
    let report = try ledger.report(month: "2026-11")
    #expect(report.map(\.category) == ["Food", "Travel"])
    #expect(report[0].remaining == -250)
    #expect(report[1].spent == 0)
    #expect(try ledger.filtered(month: "2026-10").count == 1)
    #expect(throws: LedgerError.self) { try ledger.report(month: "2026-13") }
}

@Test func totalsCannotOverflowAndRemoveIsExplicit() throws {
    var ledger = try Ledger()
    try ledger.importRows([row(amount: "92233720368547758.07")])
    let before = ledger
    #expect(throws: LedgerError.self) { try ledger.importRows([row("overflow", amount: "0.01")]) }
    #expect(ledger == before)
    #expect(throws: LedgerError.self) { try ledger.remove(reference: "missing") }
    try ledger.remove(reference: "receipt-1")
    #expect(ledger.entries.isEmpty)
}

@Test func datesAndTerminalTextAreValidated() throws {
    var ledger = try Ledger()
    try ledger.importRows([row(date: "2024-02-29")])
    for date in ["2025-02-29", "2026-2-01", "2026-00-01", "2026-11-00", "2026-11-12junk"] {
        #expect(throws: LedgerError.self) { try ledger.importRows([row("bad", date: date)]) }
    }
    #expect(throws: LedgerError.self) {
        try ledger.importRows([row("bad", category: "Food\u{1B}[2J")])
    }
}

@Test func fileTransactionsPersistAndFailuresPreserveBytes() throws {
    let directory =
        (ProcessInfo.processInfo.environment["TMPDIR"].map {
            URL(fileURLWithPath: $0, isDirectory: true)
        } ?? FileManager.default.temporaryDirectory).appendingPathComponent(UUID().uuidString)
    defer { try? FileManager.default.removeItem(at: directory) }
    let file = LedgerFile(url: directory.appendingPathComponent("ledger.json"))
    _ = try file.transaction { try $0.importRows([row()]) }
    let before = try Data(contentsOf: file.url)
    #expect(try file.transaction { $0.entries.count } == 1)
    #expect(throws: LedgerError.self) {
        try file.transaction { ledger in
            try ledger.importRows([row("new")])
            throw LedgerError.invalid("cancel")
        }
    }
    #expect(try Data(contentsOf: file.url) == before)
    try Data("broken".utf8).write(to: file.url)
    #expect(throws: DecodingError.self) { try file.transaction { try $0.importRows([row()]) } }
    #expect(try Data(contentsOf: file.url) == Data("broken".utf8))
}

@Test func duplicateOrFutureSchemaCannotLoad() throws {
    var ledger = try Ledger()
    try ledger.importRows([row()])
    let file = LedgerFile(url: URL(fileURLWithPath: "/unused"))
    let data = try file.encoded(ledger)
    var object = try #require(JSONSerialization.jsonObject(with: data) as? [String: Any])
    object["schemaVersion"] = 99
    let invalid = try JSONDecoder().decode(
        Ledger.self, from: JSONSerialization.data(withJSONObject: object))
    #expect(throws: LedgerError.self) { try invalid.validated() }
    object["schemaVersion"] = 1
    let entries = try #require(object["entries"] as? [[String: Any]])
    object["entries"] = entries + entries
    let duplicate = try JSONDecoder().decode(
        Ledger.self, from: JSONSerialization.data(withJSONObject: object))
    #expect(throws: LedgerError.self) { try duplicate.validated() }
}
