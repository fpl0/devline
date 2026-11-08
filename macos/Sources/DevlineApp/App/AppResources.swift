import Foundation

extension Bundle {
    static var devlineResources: Bundle {
        Bundle.main.url(forResource: "DevlineMac_DevlineApp", withExtension: "bundle")
            .flatMap(Bundle.init(url:)) ?? Bundle.module
    }
}
