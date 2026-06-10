package io.joern.rust2cpg.parser

import io.joern.rust2cpg.parser.RustNodeSyntax.{
  ArrayExpr,
  BlockExpr,
  CallExpr,
  ExprStmt,
  Fn,
  FormatArgsExpr,
  Literal,
  MacroExpr,
  MacroStmts
}
import org.scalatest.Inside.inside
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

  test("stringify! expansion") {
    val snippet =
      """
        |fn main() {
        | let s = stringify!(hello world);
        |}
        |""".stripMargin

    val srcFile = code(snippet, noSysRoot = false)

    srcFile.prettyPrint shouldBe
      """SOURCE_FILE
        |  FN
        |    FN_KW
        |    NAME
        |      IDENT
        |    PARAM_LIST
        |      L_PAREN
        |      R_PAREN
        |    BLOCK_EXPR
        |      STMT_LIST
        |        L_CURLY
        |        LET_STMT
        |          LET_KW
        |          IDENT_PAT
        |            NAME
        |              IDENT
        |          EQ
        |          MACRO_EXPR
        |            MACRO_CALL
        |              PATH
        |                PATH_SEGMENT
        |                  NAME_REF
        |                    IDENT
        |              BANG
        |              TOKEN_TREE
        |                L_PAREN
        |                IDENT
        |                IDENT
        |                R_PAREN
        |          SEMICOLON
        |        R_CURLY""".stripMargin

    inside(macroCalls(srcFile)) {
      case stringify :: Nil =>
        stringify.textFrom(snippet) shouldBe "stringify!(hello world)"
        inside(stringify.macroExpansion) {
          case Some(lit: Literal) =>
            lit.typeFullName shouldBe Some("&str")
            lit.textFrom(snippet) shouldBe "\"hello world\""
        }
    }
  }

  test("vec! and println! expansion") {
    val snippet =
      """
        |fn main() {
        | let v = vec![1, 2, 3];
        | println!("{:?}", v);
        |}
        |""".stripMargin
    val srcFile = code(snippet, noSysRoot = false)

    inside(macroCalls(srcFile)) {
      case vec :: print :: Nil =>
        vec.textFrom(snippet) shouldBe "vec![1, 2, 3]"
        print.textFrom(snippet) shouldBe "println!(\"{:?}\", v)"

        inside(vec.macroExpansion) {
          case Some(call: CallExpr) =>
            call.textFrom(snippet) shouldBe "alloc::boxed::box_assume_init_into_vec_unsafe(alloc::intrinsics::write_box_via_move(alloc::boxed::Box::new_uninit(),[1,2,3]))"
            call.methodFullName shouldBe Some("alloc::boxed::box_assume_init_into_vec_unsafe<T, N>")
            call.typeFullName shouldBe Some("alloc::vec::Vec<i32, alloc::alloc::Global>")
            inside(call.argList.expr) {
              case (writeBoxCall: CallExpr) :: Nil =>
                writeBoxCall.textFrom(snippet) shouldBe "alloc::intrinsics::write_box_via_move(alloc::boxed::Box::new_uninit(),[1,2,3])"
                writeBoxCall.methodFullName shouldBe Some("alloc::intrinsics::write_box_via_move<T>")
                writeBoxCall.typeFullName shouldBe Some("alloc::boxed::Box<core::mem::maybe_uninit::MaybeUninit<[i32; 3]>, alloc::alloc::Global>")
                inside(writeBoxCall.argList.expr) {
                  case (uninitCall: CallExpr) :: (array: ArrayExpr) :: Nil =>
                    uninitCall.textFrom(snippet) shouldBe "alloc::boxed::Box::new_uninit()"
                    uninitCall.methodFullName shouldBe Some("alloc::boxed::Box<T, alloc::alloc::Global>::new_uninit")
                    uninitCall.typeFullName shouldBe Some("alloc::boxed::Box<core::mem::maybe_uninit::MaybeUninit<[i32; 3]>, alloc::alloc::Global>")
                    inside(array.expr) {
                      case (_: Literal) :: (two: Literal) :: (_: Literal) :: Nil =>
                        two.textFrom(snippet) shouldBe "2"
                        two.typeFullName shouldBe Some("i32")
                    }
                }
            }
        }

        inside(print.macroExpansion) {
          case Some(macroStmts: MacroStmts) =>
            macroStmts.stmt shouldBe Nil
            inside(macroStmts.expr) {
              case Some(blockExpr: BlockExpr) =>
                inside(blockExpr.stmtList.stmt) {
                  case (exprStmt: ExprStmt) :: Nil =>
                    inside(exprStmt.expr) {
                      case callExpr: CallExpr =>
                        callExpr.methodFullName shouldBe Some("std::io::stdio::_print")
                        callExpr.typeFullName shouldBe Some("()")
                        callExpr.textFrom(snippet) shouldBe "std::io::_print(std::format_args_nl!(\"{:?}\",v))"
                        inside(callExpr.argList.expr) {
                          case (formatArgs: MacroExpr) :: Nil =>
                            inside(formatArgs.macroCall.macroExpansion) {
                              case Some(formatArgsExpr: FormatArgsExpr) =>
                                inside(formatArgsExpr.formatArgsArg) {
                                  case formatArgsArg :: Nil =>
                                    val arg = formatArgsArg.expr
                                    arg.textFrom(snippet) shouldBe "v"
                                    arg.typeFullName shouldBe Some("alloc::vec::Vec<i32, alloc::alloc::Global>")
                                    arg.methodFullName shouldBe None
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    srcFile.prettyPrint shouldBe
      """SOURCE_FILE
        |  FN
        |    FN_KW
        |    NAME
        |      IDENT
        |    PARAM_LIST
        |      L_PAREN
        |      R_PAREN
        |    BLOCK_EXPR
        |      STMT_LIST
        |        L_CURLY
        |        LET_STMT
        |          LET_KW
        |          IDENT_PAT
        |            NAME
        |              IDENT
        |          EQ
        |          MACRO_EXPR
        |            MACRO_CALL
        |              PATH
        |                PATH_SEGMENT
        |                  NAME_REF
        |                    IDENT
        |              BANG
        |              TOKEN_TREE
        |                L_BRACK
        |                INT_NUMBER
        |                COMMA
        |                INT_NUMBER
        |                COMMA
        |                INT_NUMBER
        |                R_BRACK
        |          SEMICOLON
        |        EXPR_STMT
        |          MACRO_EXPR
        |            MACRO_CALL
        |              PATH
        |                PATH_SEGMENT
        |                  NAME_REF
        |                    IDENT
        |              BANG
        |              TOKEN_TREE
        |                L_PAREN
        |                STRING
        |                COMMA
        |                IDENT
        |                R_PAREN
        |          SEMICOLON
        |        R_CURLY""".stripMargin

  }

  test("multi-statement expansion is rendered with whitespace") {
    val snippet =
      """
        |macro_rules! two_lets {
        |    ($a:expr) => {
        |        let p = $a;
        |        let q = p + 1;
        |    };
        |}
        |
        |fn main() {
        |    two_lets!(4);
        |}
        |""".stripMargin
    val srcFile = code(snippet)

    val main = srcFile.item.collectFirst { case fn: Fn => fn }.get
    inside(macroCalls(main)) {
      case twoLets :: Nil =>
        inside(twoLets.macroExpansion) {
          case Some(macroStmts: MacroStmts) =>
            macroStmts.textFrom(snippet) shouldBe "let p = 4;\nlet q = p+1;"
        }
    }
  }
}
