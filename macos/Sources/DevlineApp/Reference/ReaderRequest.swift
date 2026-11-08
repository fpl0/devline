import Foundation

struct ReaderSection: Equatable, Identifiable {
    enum Target: Equatable {
        case text(NSRange)
        case anchor(String)
    }

    let title: String
    let target: Target
    var depth = 2
    var fallbackText: String?
    var id: String {
        switch target {
        case .text(let range): "text-\(range.location)"
        case .anchor(let id): id
        }
    }
}

struct ReaderRequest: Equatable {
    enum Action: Equatable {
        case find(text: String, backwards: Bool, fromStart: Bool)
        case focus(String)
        case jump(ReaderSection)
        case restore(Double, anchor: String?, offset: Double?)
    }

    let id = UUID()
    let action: Action
    var cue = false
}
