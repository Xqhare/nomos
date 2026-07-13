import XCTest
import SwiftTreeSitter
import TreeSitterNomos

final class TreeSitterNomosTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_nomos())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Nomos grammar")
    }
}
