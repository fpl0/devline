import AppKit
let input = CommandLine.arguments[1]
let darkInput = CommandLine.arguments[2]
let output = CommandLine.arguments[3]
guard let lightIcon = NSImage(contentsOfFile: input), let darkIcon = NSImage(contentsOfFile: darkInput) else { fatalError("Cannot read native icon renders") }
let width = 1000, height = 430
let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
for (column, dark) in [false,true].enumerated() {
 let icon = dark ? darkIcon : lightIcon
 let x = CGFloat(column * 500)
 let background = dark ? NSColor(calibratedWhite: 0.10, alpha: 1) : NSColor(calibratedWhite: 0.98, alpha: 1)
 background.setFill()
 NSRect(x:x, y:0, width:500, height:430).fill()
 let text = dark ? NSColor.white : NSColor.black
 let attributes: [NSAttributedString.Key: Any] = [.font:NSFont.systemFont(ofSize:20,weight:.medium),.foregroundColor:text]
 (dark ? "Native dark appearance" : "Native light appearance" as NSString).draw(at:NSPoint(x:x+26,y:382),withAttributes:attributes)
 icon.draw(in:NSRect(x:x+132,y:136,width:236,height:236))
 for (i,size) in [16,32,64,128].enumerated() {
  let center = x + CGFloat(62+i*125)
  icon.draw(in:NSRect(x:center-CGFloat(size)/2,y:35,width:CGFloat(size),height:CGFloat(size)))
  ("\(size) px" as NSString).draw(at:NSPoint(x:center-20,y:12),withAttributes:[.font:NSFont.monospacedSystemFont(ofSize:11,weight:.regular),.foregroundColor:text.withAlphaComponent(0.7)])
 }
}
NSGraphicsContext.restoreGraphicsState()
try bitmap.representation(using:.png,properties:[:])!.write(to:URL(fileURLWithPath:output))
