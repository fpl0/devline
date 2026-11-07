import Foundation

struct JobOutcome: Decodable, Sendable {
    struct Step: Decodable, Sendable {
        let id, name, kind, status, state, detail: String?
        var needsAttention: Bool { ["failed", "blocked", "needs_attention", "error", "unchecked"].contains(state ?? status ?? "") }
    }
    let complete, readyOffline, allChecked: Bool?
    let status: String?
    let steps, sources, blockers: [Step]?
    private enum CodingKeys: String, CodingKey { case complete, readyOffline, allChecked, status, steps, sources, blockers }
    init(from decoder: Decoder) throws {
        let values = try? decoder.container(keyedBy: CodingKeys.self)
        complete = try? values?.decodeIfPresent(Bool.self, forKey: .complete)
        readyOffline = try? values?.decodeIfPresent(Bool.self, forKey: .readyOffline)
        allChecked = try? values?.decodeIfPresent(Bool.self, forKey: .allChecked)
        status = try? values?.decodeIfPresent(String.self, forKey: .status)
        steps = try? values?.decodeIfPresent([Step].self, forKey: .steps)
        sources = try? values?.decodeIfPresent([Step].self, forKey: .sources)
        blockers = try? values?.decodeIfPresent([Step].self, forKey: .blockers)
    }
    var needsAttention: Bool { complete == false || readyOffline == false || allChecked == false || ["needs_attention", "needs_preparation"].contains(status ?? "") }
    var issues: [Step] { (steps ?? []).filter(\.needsAttention) + (sources ?? []).filter(\.needsAttention) + (blockers ?? []) }
}

struct JobRecord: Decodable, Identifiable, Sendable {
    let id, kind, state, stage: String
    let createdAt, updatedAt: Double
    let error, logPath, log: String?
    let targetPath, targetReference: String?
    let result: JobOutcome?
    let offset, nextOffset, completedBytes, totalBytes: Int?
    var needsAttention: Bool { state == "failed" || (!isActive && result?.needsAttention == true) }
    var stateLabel: String { needsAttention ? "Needs Attention" : state.capitalized }
    var isActive: Bool { state == "queued" || state == "running" }
    var title: String {
        switch kind {
        case "build": "Refresh Starter Dependencies"
        case "verify": "Verify Offline Readiness"
        case "prepare": "Prepare Starter"
        case "new_project": "Create Offline Project"
        case "project_prepare": "Prepare Project Dependencies"
        case "toolbox_prepare": "Prepare Project for Offline Work"
        case "content_check": "Check All Content Updates"
        case "content_update": "Update Prepared Content"
        case "infra_prepare": "Prepare Image Archive"
        case "infra_restore": "Restore Image to Docker"
        case "infra_verify": "Verify Image Archive"
        case "infra_update": "Update Image Archive"
        case "infra_stack_up": "Start Project Services"
        case "infra_stack_down": "Stop Project Services"
        case "infra_stack_add": "Add Project Services and Clients"
        case "service_clients_add": "Add Project Client Libraries"
        case "infra_export": "Export Image Archives"
        case "infra_import": "Import Image Archives"
        case "docs_prepare": "Prepare Local Documentation"
        case "docs_install": "Download Documentation"
        case "bundle_export": "Export Offline Bundle"
        case "bundle_import": "Import Offline Bundle"
        default: kind.capitalized
        }
    }
    var icon: String { isActive ? "arrow.triangle.2.circlepath" : needsAttention ? "exclamationmark.circle" : state == "succeeded" ? "checkmark.circle" : state == "cancelled" ? "stop.circle" : "exclamationmark.circle" }
}
