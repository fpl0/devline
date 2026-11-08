import Foundation

func safeExternalURL(_ value: String) -> URL? {
    guard let url = URL(string: value), let scheme = url.scheme?.lowercased(),
          scheme == "https" || scheme == "http", url.host != nil,
          url.user == nil, url.password == nil else { return nil }
    return url
}
