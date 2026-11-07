import Foundation
import PostgresNIO
import MySQLNIO
import BunnySwift
import RediStack
import MongoKitten
import Nats
import MeiliSearch
import SotoS3
import AppAuth
print("PASS swift:postgres \(String(reflecting: PostgresConnection.self))")
print("PASS swift:mysql \(String(reflecting: MySQLConnection.self))")
print("PASS swift:rabbitmq \(String(reflecting: BunnySwift.Connection.self))")
print("PASS swift:redis \(String(reflecting: RedisConnection.self))")
print("PASS swift:mongodb \(String(reflecting: MongoDatabase.self))")
print("PASS swift:nats \(String(reflecting: NatsClient.self))")
print("PASS swift:meilisearch \(String(reflecting: MeiliSearch.self))")
print("PASS swift:seaweedfs \(String(reflecting: S3.self))")
print("PASS swift:keycloak \(String(reflecting: OIDServiceConfiguration.self))")
var mailRequest = URLRequest(url: URL(string: "http://127.0.0.1:8025/api/v1/send")!)
mailRequest.httpMethod = "POST"
mailRequest.httpBody = try JSONSerialization.data(withJSONObject: ["Subject": "Offline proof", "Text": "A local test message"])
precondition(mailRequest.httpBody != nil)
print("PASS swift:mailpit Foundation HTTP request")
