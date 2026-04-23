package io.joern.rust2cpg.parser

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers.shouldBe

class RustAstGenPrettyPrintTests extends AnyFunSuite with RustAstGenTestFixture {

  test("fn sum") {
    val srcFile = code(
      """fn sum(a: i32, b: i32) -> i32 {
        |    let total = a + b;
        |    total
        |}
        |""".stripMargin)

    srcFile.prettyPrint shouldBe
      """SOURCE_FILE
        |  FN
        |    FN_KW
        |    NAME
        |      IDENT
        |    PARAM_LIST
        |      L_PAREN
        |      PARAM
        |        IDENT_PAT
        |          NAME
        |            IDENT
        |        COLON
        |        PATH_TYPE
        |          PATH
        |            PATH_SEGMENT
        |              NAME_REF
        |                IDENT
        |      COMMA
        |      PARAM
        |        IDENT_PAT
        |          NAME
        |            IDENT
        |        COLON
        |        PATH_TYPE
        |          PATH
        |            PATH_SEGMENT
        |              NAME_REF
        |                IDENT
        |      R_PAREN
        |    RET_TYPE
        |      THIN_ARROW
        |      PATH_TYPE
        |        PATH
        |          PATH_SEGMENT
        |            NAME_REF
        |              IDENT
        |    BLOCK_EXPR
        |      STMT_LIST
        |        L_CURLY
        |        LET_STMT
        |          LET_KW
        |          IDENT_PAT
        |            NAME
        |              IDENT
        |          EQ
        |          BIN_EXPR
        |            PATH_EXPR
        |              PATH
        |                PATH_SEGMENT
        |                  NAME_REF
        |                    IDENT
        |            PLUS
        |            PATH_EXPR
        |              PATH
        |                PATH_SEGMENT
        |                  NAME_REF
        |                    IDENT
        |          SEMICOLON
        |        PATH_EXPR
        |          PATH
        |            PATH_SEGMENT
        |              NAME_REF
        |                IDENT
        |        R_CURLY""".stripMargin
  }

  test("pub struct with single private field") {
    val srcFile = code(
      """
        |pub struct Foo {
        | my_field: i32,
        |}
        |""".stripMargin)

    srcFile.prettyPrint shouldBe
      """SOURCE_FILE
        |  STRUCT
        |    VISIBILITY
        |      PUB_KW
        |    STRUCT_KW
        |    NAME
        |      IDENT
        |    RECORD_FIELD_LIST
        |      L_CURLY
        |      RECORD_FIELD
        |        NAME
        |          IDENT
        |        COLON
        |        PATH_TYPE
        |          PATH
        |            PATH_SEGMENT
        |              NAME_REF
        |                IDENT
        |      COMMA
        |      R_CURLY""".stripMargin

  }

}
