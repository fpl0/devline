import Darwin
import Foundation

// The demo needs only a small subset of the Redis and NATS text protocols.
final class LoopbackConnection {
    private let fd: Int32
    private let service: String

    init(_ service: String, port: Int) throws {
        let fd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)
        guard fd >= 0 else { throw DispatchError("Could not open the \(service) socket.") }
        var timeout = timeval(tv_sec: 5, tv_usec: 0)
        var noSignal: Int32 = 1
        _ = setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, socklen_t(MemoryLayout<timeval>.size))
        _ = setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, socklen_t(MemoryLayout<timeval>.size))
        _ = setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &noSignal, socklen_t(MemoryLayout<Int32>.size))
        var address = sockaddr_in()
        address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        address.sin_family = sa_family_t(AF_INET)
        address.sin_port = UInt16(port).bigEndian
        address.sin_addr.s_addr = inet_addr("127.0.0.1")
        let connected = withUnsafePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { Darwin.connect(fd, $0, socklen_t(MemoryLayout<sockaddr_in>.size)) }
        }
        guard connected == 0 else {
            Darwin.close(fd)
            throw DispatchError("Could not connect to local \(service).")
        }
        self.fd = fd
        self.service = service
    }

    deinit { Darwin.close(fd) }

    func send(_ data: Data) throws {
        try data.withUnsafeBytes { buffer in
            guard let pointer = buffer.baseAddress else { return }
            var written = 0
            while written < buffer.count {
                let count = Darwin.send(fd, pointer.advanced(by: written), buffer.count - written, 0)
                guard count > 0 else { throw DispatchError("Could not send to \(service).") }
                written += count
            }
        }
    }

    func read(_ count: Int) throws -> Data {
        var data = Data(count: count)
        try data.withUnsafeMutableBytes { buffer in
            guard let pointer = buffer.baseAddress else { return }
            var consumed = 0
            while consumed < count {
                let received = recv(fd, pointer.advanced(by: consumed), count - consumed, 0)
                guard received > 0 else { throw DispatchError("Timed out or disconnected while reading from \(service).") }
                consumed += received
            }
        }
        return data
    }

    func line() throws -> String {
        var line = Data()
        for _ in 0..<8_192 {
            let byte = try read(1)[0]
            if byte == 10, line.last == 13 {
                return String(decoding: line.dropLast(), as: UTF8.self)
            }
            line.append(byte)
        }
        throw DispatchError("\(service) sent an overlong response.")
    }

    func redisCommand(_ parts: [Data]) throws {
        var command = Data("*\(parts.count)\r\n".utf8)
        for part in parts {
            command.append(Data("$\(part.count)\r\n".utf8))
            command.append(part)
            command.append(Data("\r\n".utf8))
        }
        try send(command)
    }
}
