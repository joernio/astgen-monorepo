import better.files.{File => BFile}
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import BabelNodeSyntax.*
import ujson.Value

import java.lang.reflect.{InvocationTargetException, Modifier}
import java.util.concurrent.ConcurrentLinkedQueue
import scala.jdk.CollectionConverters.*
import scala.sys.process.*

/** Round-trip tests that exercise the generated `BabelNodeSyntax` Scala API against the real
  * `astgen` CLI.
  *
  * The binary is located via the `ASTGEN_BIN` environment variable, or else by its release name
  * (`astgen-linux`, `astgen-macos`, ...) inside `javascript-astgen/`; see `scala-tests/README.md`.
  */
class BabelNodeSyntaxTest extends AnyWordSpec with Matchers {

  private val shellPrefix: Seq[String] =
    if (scala.util.Properties.isWin) "cmd" :: "/c" :: Nil else "sh" :: "-c" :: Nil

  private val executableName: String = {
    val arm = System.getProperty("os.arch").toLowerCase.matches("aarch64|arm64")
    if (scala.util.Properties.isWin) "astgen-win.exe"
    else if (scala.util.Properties.isMac) { if (arm) "astgen-macos-arm" else "astgen-macos" }
    else { if (arm) "astgen-linux-arm" else "astgen-linux" }
  }

  /** Absolute path of the `astgen` binary under test. */
  private def executablePath: String =
    sys.env.getOrElse(
      "ASTGEN_BIN",
      (BFile(".").parent / executableName).toJava.toPath.normalize.toAbsolutePath.toString
    )

  private def fixtureDir: BFile = BFile(".").parent / "test" / "fixtures" / "syntax"

  /** Runs `command` in `cwd` and asserts a zero exit code. */
  private def run(command: String, cwd: String): Unit = {
    val stdOut = new ConcurrentLinkedQueue[String]
    val stdErr = new ConcurrentLinkedQueue[String]
    val logger = ProcessLogger(stdOut.add, stdErr.add)
    val exit   = Process(shellPrefix :+ command, new java.io.File(cwd)).!(logger)
    if (exit != 0) {
      val combined =
        s"""--- stdout ---
           |${stdOut.asScala.mkString(System.lineSeparator())}
           |--- stderr ---
           |${stdErr.asScala.mkString(System.lineSeparator())}""".stripMargin
      fail(s"Command `$command` (cwd=$cwd) exited with $exit\n$combined")
    }
  }

  /** Creates a temporary project from `sources` (file name -> content), runs `astgen --tsTypes=false`
    * on it, and passes the raw JSON of each file (keyed by source file name) to `body`.
    */
  private def withProject(sources: Map[String, String])(body: Map[String, Value] => Unit): Unit = {
    val project: BFile = BFile.newTemporaryDirectory("babelastgentests")
    try {
      sources.foreach { case (name, content) => (project / name).createIfNotExists(createParents = true).write(content) }
      run(s"\"$executablePath\" --tsTypes=false", project.pathAsString)
      val asts = sources.keys.map { name =>
        name -> ujson.read((project / "ast_out" / s"$name.json").contentAsString)
      }.toMap
      body(asts)
    } finally {
      project.delete(swallowIOExceptions = true)
    }
  }

  /** Single-file convenience: the decoded root `File` node. */
  private def withFile(name: String, source: String)(body: File => Unit): Unit =
    withProject(Map(name -> source)) { asts =>
      body(BabelAstFile(asts(name)).ast)
    }

  /** Every JSON object that has a string `type`, in document order. */
  private def allNodeJson(value: Value): Iterator[Value] = value match {
    case obj: ujson.Obj =>
      val self  = if (obj.value.get("type").exists(_.strOpt.isDefined) && obj.value.contains("start")) Iterator.single(obj: Value) else Iterator.empty
      val below = obj.value.iterator.filterNot(_._1 == "loc").flatMap { case (_, v) => allNodeJson(v) }
      self ++ below
    case arr: ujson.Arr => arr.value.iterator.flatMap(allNodeJson)
    case _              => Iterator.empty
  }

  /** The generated field accessors of `node`. */
  private def accessorsOf(node: BabelNode): Seq[java.lang.reflect.Method] = {
    val skip = Set("json", "hashCode", "toString", "productArity", "productPrefix", "productIterator", "productElementNames")
    node.getClass.getDeclaredMethods.toSeq
      .filter(m => m.getParameterCount == 0 && !Modifier.isPrivate(m.getModifiers) && !Modifier.isStatic(m.getModifiers))
      .filter(m => m.getReturnType != Void.TYPE && !m.getName.contains("$") && !m.getName.startsWith("_") && !skip(m.getName))
  }

  /** Invokes every generated field accessor of `node`, failing with the offending `Type.field` if one throws. */
  private def forceAccessors(node: BabelNode): Unit =
    accessorsOf(node).foreach { m =>
      try m.invoke(node)
      catch {
        case e: InvocationTargetException =>
          fail(s"${node.nodeType}.${m.getName} threw ${e.getCause}")
      }
    }

  /** Every node object together with the node object that contains it. */
  private def nodesWithParent(value: Value, parent: Option[ujson.Obj] = None): Iterator[(Option[ujson.Obj], ujson.Obj)] = value match {
    case obj: ujson.Obj =>
      val isNode = obj.value.get("type").exists(_.strOpt.isDefined) && obj.value.contains("start")
      val self   = if (isNode) Iterator.single(parent -> obj) else Iterator.empty
      val below  = obj.value.iterator.filterNot(_._1 == "loc").flatMap { case (_, v) => nodesWithParent(v, if (isNode) Some(obj) else parent) }
      self ++ below
    case arr: ujson.Arr => arr.value.iterator.flatMap(nodesWithParent(_, parent))
    case _              => Iterator.empty
  }

  "BabelNodeSyntax" should {

    "decode a JavaScript module with typed accessors" in {
      val source =
        """import def, {a as b} from "mod"
          |export class Foo extends Bar {
          |  static count = 1
          |  #secret = 2
          |  constructor(x, y = 2) { super(x); this.x = x }
          |  async run(...args) { return await this.#secret + args.length }
          |}
          |""".stripMargin
      withFile("main.js", source) { file =>
        file.program.sourceType shouldBe "module"
        val Seq(importDecl, exportDecl) = file.program.body

        val imp = importDecl.asInstanceOf[ImportDeclaration]
        imp.source.value shouldBe "mod"
        imp.specifiers.map(_.nodeType) shouldBe Seq("ImportDefaultSpecifier", "ImportSpecifier")
        imp.specifiers.last.asInstanceOf[ImportSpecifier].imported.asInstanceOf[Identifier].name shouldBe "a"

        val cls = exportDecl.asInstanceOf[ExportNamedDeclaration].declaration.get.asInstanceOf[ClassDeclaration]
        cls.id.get.name shouldBe "Foo"
        cls.superClass.get.asInstanceOf[Identifier].name shouldBe "Bar"

        val members = cls.body.body
        members.map(_.nodeType) shouldBe Seq("ClassProperty", "ClassPrivateProperty", "ClassMethod", "ClassMethod")
        members.head.asInstanceOf[ClassProperty].static shouldBe true

        val ctor = members(2).asInstanceOf[ClassMethod]
        ctor.kind shouldBe "constructor"
        ctor.params.map(_.nodeType) shouldBe Seq("Identifier", "AssignmentPattern")

        val run = members(3).asInstanceOf[ClassMethod]
        run.async shouldBe true
        run.params.map(_.nodeType) shouldBe Seq("RestElement")
      }
    }

    "model optional fields as Option and arrays as Seq" in {
      withFile("main.js", "let a;\nconst b = [1, , 3];\nfunction f() {}") { file =>
        val Seq(a, b, f) = file.program.body
        a.asInstanceOf[VariableDeclaration].declarations.head.init shouldBe None
        a.asInstanceOf[VariableDeclaration].kind shouldBe "let"

        val elements = b.asInstanceOf[VariableDeclaration].declarations.head.init.get.asInstanceOf[ArrayExpression].elements
        elements.map(_.map(_.nodeType)) shouldBe Seq(Some("NumericLiteral"), None, Some("NumericLiteral"))

        f.asInstanceOf[FunctionDeclaration].params shouldBe empty
        file.program.directives shouldBe empty
      }
    }

    "expose literals, raw text and parentheses" in {
      withFile("main.js", "const s = 'x', n = (42), t = `a${s}b`;") { file =>
        val decls = file.program.body.head.asInstanceOf[VariableDeclaration].declarations
        val str   = decls(0).init.get.asInstanceOf[StringLiteral]
        str.value shouldBe "x"
        str.extra("raw").str shouldBe "'x'"
        str.isParenthesized shouldBe false

        val num = decls(1).init.get.asInstanceOf[NumericLiteral]
        num.value shouldBe Some(42.0)
        num.isParenthesized shouldBe true

        val tpl = decls(2).init.get.asInstanceOf[TemplateLiteral]
        tpl.quasis.size shouldBe 2
        tpl.expressions.map(_.nodeType) shouldBe Seq("Identifier")
        tpl.quasis.head.tail shouldBe false
      }
    }

    "decode TypeScript declarations" in {
      val source =
        """interface Shape<T extends object = {}> { readonly name: string; area?(): number }
          |abstract class Base<T> implements Shape<T> {
          |  private readonly x: number = 1
          |  constructor(public y: string) { super() }
          |  abstract area(): number
          |}
          |enum Color { Red = 1, Green }
          |type Fn = (a: number) => string
          |""".stripMargin
      withFile("main.ts", source) { file =>
        val Seq(iface, cls, enm, alias) = file.program.body

        val shape = iface.asInstanceOf[TSInterfaceDeclaration]
        shape.id.name shouldBe "Shape"
        shape.typeParameters.get.params.head.name shouldBe "T"
        shape.body.body.map(_.nodeType) shouldBe Seq("TSPropertySignature", "TSMethodSignature")
        shape.body.body.head.asInstanceOf[TSPropertySignature].readonly shouldBe Some(true)

        val base = cls.asInstanceOf[ClassDeclaration]
        base.`abstract` shouldBe Some(true)
        base.implements.size shouldBe 1
        val x = base.body.body.head.asInstanceOf[ClassProperty]
        x.accessibility shouldBe Some("private")
        x.readonly shouldBe Some(true)
        x.typeAnnotation.get.asInstanceOf[TSTypeAnnotation].typeAnnotation.nodeType shouldBe "TSNumberKeyword"

        val ctorParam = base.body.body(1).asInstanceOf[ClassMethod].params.head.asInstanceOf[TSParameterProperty]
        ctorParam.accessibility shouldBe Some("public")

        enm.asInstanceOf[TSEnumDeclaration].members.map(_.id.asInstanceOf[Identifier].name) shouldBe Seq("Red", "Green")
        alias.asInstanceOf[TSTypeAliasDeclaration].typeAnnotation.nodeType shouldBe "TSFunctionType"
      }
    }

    "decode JSX in a .tsx file" in {
      withFile("main.tsx", "const el = <div className=\"x\" {...rest}>hi {name}</div>;") { file =>
        val el = file.program.body.head.asInstanceOf[VariableDeclaration].declarations.head.init.get.asInstanceOf[JSXElement]
        el.openingElement.name.asInstanceOf[JSXIdentifier].name shouldBe "div"
        el.openingElement.attributes.map(_.nodeType) shouldBe Seq("JSXAttribute", "JSXSpreadAttribute")
        el.children.map(_.nodeType) shouldBe Seq("JSXText", "JSXExpressionContainer")
        el.closingElement.map(_.name.asInstanceOf[JSXIdentifier].name) shouldBe Some("div")
      }
    }

    "expose comments, offsets and line/column information" in {
      withFile("main.js", "// lead\nfoo(); /* trail */\n") { file =>
        val stmt = file.program.body.head
        stmt.leadingComments.map(_.nodeType) shouldBe Seq("CommentLine")
        stmt.leadingComments.head.asInstanceOf[CommentLine].value shouldBe " lead"
        stmt.trailingComments.map(_.nodeType) shouldBe Seq("CommentBlock")
        file.comments.size shouldBe 2

        stmt.start shouldBe Some(8)
        stmt.end shouldBe Some(14)
        stmt.startLine shouldBe Some(2)
        stmt.startColumn shouldBe Some(0)
        stmt.endLine shouldBe Some(2)
        stmt.endColumn shouldBe Some(6)
      }
    }

    "keep complete positions on every node (shared Position objects must not be dropped)" in {
      withProject(Map("es.js" -> (fixtureDir / "es.js.fixture").contentAsString)) { asts =>
        val nodes = allNodeJson(asts("es.js")("ast")).toSeq
        nodes.size should be > 100
        val incomplete = nodes.flatMap { n =>
          val loc = n.obj.get("loc").map(_.obj)
          val ok = loc.exists(l => l.get("start").exists(_.obj.contains("line")) && l.get("end").exists(_.obj.contains("line")))
          if (ok) None else Some(n("type").str + "@" + n("start").num.toInt)
        }
        incomplete shouldBe empty
      }
    }

    "report recovered parse errors on the file node" in {
      withFile("main.js", "let a = 1;\nlet a = 2;\n") { file =>
        file.errors should not be empty
        file.errors.head("reasonCode").str shouldBe "VarRedeclaration"
      }
    }

    "decode the BabelAstFile envelope" in {
      withProject(Map("sub/dir/main.js" -> "1;")) { asts =>
        val envelope = BabelAstFile(asts("sub/dir/main.js"))
        envelope.relativeName.replace('\\', '/') shouldBe "sub/dir/main.js"
        envelope.fullName should endWith("main.js")
        envelope.ast.program.body.size shouldBe 1
      }
    }

    "decode unknown node types to UnknownNode instead of failing" in {
      val node = createBabelNode(ujson.Obj("type" -> "DefinitelyNotANode", "start" -> 3, "end" -> 7))
      node shouldBe a[UnknownNode]
      node.nodeType shouldBe "DefinitelyNotANode"
      node.start shouldBe Some(3)
    }

    "keep the rest of a file usable when it contains an unknown node" in {
      val json = ujson.read(
        """{"type":"File","program":{"type":"Program","sourceType":"module","body":[
          |  {"type":"FromTheFuture","start":0,"end":1},
          |  {"type":"EmptyStatement"}
          |]}}""".stripMargin
      )
      val body = createBabelNode(json).asInstanceOf[File].program.body
      body.map(_.nodeType) shouldBe Seq("FromTheFuture", "EmptyStatement")
      body.head shouldBe a[UnknownNode]
      // it stands in for an alias such as Statement, so no cast fails
      body.head.isInstanceOf[Statement] shouldBe true
    }

    "decode a module declaration without a body" in {
      withFile("main.ts", """declare module "ambient"; namespace A.B {}""") { file =>
        val Seq(ambient, ns) = file.program.body.map(_.asInstanceOf[TSModuleDeclaration])
        ambient.body shouldBe None
        ns.body should not be empty
      }
    }

    "read a number JSON cannot represent as absent, keeping the source text in extra" in {
      withFile("main.js", "var a = 1e999, b = 1;") { file =>
        val decls = file.program.body.head.asInstanceOf[VariableDeclaration].declarations
        val Seq(huge, one) = decls.map(_.init.get.asInstanceOf[NumericLiteral])
        huge.value shouldBe None
        huge.extra("raw").str shouldBe "1e999"
        one.value shouldBe Some(1.0)
      }
    }

    "keep comments that several nodes share (File.comments and the node they attach to)" in {
      withFile("main.js", "// shared\nfoo(); // after\nbar();\n") { file =>
        file.comments.map(_.asInstanceOf[CommentLine].value) shouldBe Seq(" shared", " after")
        val Seq(foo, bar) = file.program.body
        foo.leadingComments.map(_.asInstanceOf[CommentLine].value) shouldBe Seq(" shared")
        foo.trailingComments.map(_.asInstanceOf[CommentLine].value) shouldBe Seq(" after")
        bar.leadingComments.map(_.asInstanceOf[CommentLine].value) shouldBe Seq(" after")
      }
    }

    "compare nodes by the JSON object they wrap and hash them without looking inside" in {
      withFile("main.js", "a; a;") { file =>
        val Seq(first, second) = file.program.body
        file.program.body.head shouldBe first // re-wrapped on every access, still equal
        first.hashCode shouldBe file.program.body.head.hashCode
        first should not equal second // same shape, different position
        Set[BabelNode](first, second, file.program.body.head).size shouldBe 2
      }
      // hashing must not recurse into a deep subtree
      val deep = (1 to 100000).foldLeft[Value](ujson.Obj("type" -> "Identifier", "name" -> "x")) { (inner, _) =>
        ujson.Obj("type" -> "ParenthesizedExpression", "expression" -> inner)
      }
      createBabelNode(deep).hashCode should not be 0
      Set(createBabelNode(deep), createBabelNode(deep)).size shouldBe 1
    }

    "print nodes with type and position" in {
      createBabelNode(ujson.Obj("type" -> "Identifier", "name" -> "x", "start" -> 3, "end" -> 4)).toString shouldBe "Identifier(3-4)"
      createBabelNode(ujson.Obj("type" -> "Identifier", "name" -> "x")).toString shouldBe "Identifier(?-?)"
    }

    "keep extra in source order" in {
      val extra = ujson.Obj("z" -> 1, "y" -> 2, "x" -> 3, "w" -> 4, "v" -> 5, "u" -> 6)
      createBabelNode(ujson.Obj("type" -> "Identifier", "name" -> "x", "extra" -> extra)).extra.keys.toList shouldBe
        List("z", "y", "x", "w", "v", "u")
    }

    "fail with the node type and field when a node does not fit" in {
      def specifier(imported: Value): ImportSpecifier =
        createBabelNode(ujson.Obj("type" -> "ImportSpecifier", "local" -> ujson.Obj("type" -> "Identifier", "name" -> "a"), "imported" -> imported))
          .asInstanceOf[ImportSpecifier]

      // a union of concrete classes cannot hold an unknown node ...
      val unknown = intercept[IllegalStateException](specifier(ujson.Obj("type" -> "FromTheFuture")).imported)
      unknown.getMessage shouldBe "ImportSpecifier.imported: unexpected node of type 'FromTheFuture'"
      // ... nor a node of another class
      val wrong = intercept[IllegalStateException](specifier(ujson.Obj("type" -> "NullLiteral")).imported)
      wrong.getMessage should include("ImportSpecifier.imported")
      // a required field that is missing or null
      val missing = intercept[NoSuchElementException](createBabelNode(ujson.Obj("type" -> "Identifier")).asInstanceOf[Identifier].name)
      missing.getMessage shouldBe "Identifier.name: required field is missing"
      val nul = intercept[IllegalStateException](createBabelNode(ujson.Obj("type" -> "File", "program" -> ujson.Null)).asInstanceOf[File].program)
      nul.getMessage shouldBe "File.program: expected a node, found null"
      // elements of an array are checked as well
      val body = createBabelNode(ujson.Obj("type" -> "ClassBody", "body" -> ujson.Arr(ujson.Obj("type" -> "StringLiteral", "value" -> "x"))))
      intercept[IllegalStateException](body.asInstanceOf[ClassBody].body).getMessage should include("ClassBody.body")
    }

    "let an unknown node stand in wherever an alias is expected" in {
      val unknown = ujson.Obj("type" -> "FromTheFuture")
      createBabelNode(ujson.Obj("type" -> "ExpressionStatement", "expression" -> unknown)).asInstanceOf[ExpressionStatement].expression shouldBe a[UnknownNode]
      createBabelNode(ujson.Obj("type" -> "Identifier", "name" -> "x", "leadingComments" -> ujson.Arr(unknown))).leadingComments.head shouldBe a[UnknownNode]
    }

    "decode every node of every syntax fixture and evaluate all of its accessors" in {
      val fixtures = fixtureDir.list(_.name.endsWith(".fixture")).toSeq.sortBy(_.name)
      fixtures should not be empty
      // `foo.js.fixture` -> `foo.js`
      withProject(fixtures.map(f => f.name.stripSuffix(".fixture") -> f.contentAsString).toMap) { asts =>
        asts.foreach { case (name, json) =>
          val nodes = allNodeJson(json("ast")).toSeq
          nodes.size should be > 0
          nodes.foreach { n =>
            val node = createBabelNode(n)
            node.nodeType shouldBe n("type").str
            try forceAccessors(node)
            catch { case e: org.scalatest.exceptions.TestFailedException => fail(s"$name: ${e.getMessage}", e) }
          }
          // the root, too
          forceAccessors(BabelAstFile(json).ast)
        }
      }
    }

    "fail only with a descriptive error, whatever shape the parser output has" in {
      val fixtures = fixtureDir.list(_.name.endsWith(".fixture")).toSeq.sortBy(_.name)
      val random   = new scala.util.Random(20241104)
      val bad      = scala.collection.mutable.LinkedHashMap.empty[String, String]

      // What a consumer may have to handle: an unexpected node is reported as one of these two, naming Type.field.
      def documented(t: Throwable): Boolean = t.isInstanceOf[IllegalStateException] || t.isInstanceOf[NoSuchElementException]
      def record(where: String, t: Throwable): Unit =
        if (!documented(t)) bad.getOrElseUpdate(s"$where: ${t.getClass.getName}", String.valueOf(t.getMessage).take(160))

      // Evaluates the accessors of `value` and of the nodes found below it, `depth` levels deep.
      def walk(where: String, value: Any, depth: Int): Unit = value match {
        case node: BabelNode if depth >= 0 =>
          accessorsOf(node).foreach { m =>
            try walk(where, m.invoke(node), depth - 1)
            catch {
              case e: InvocationTargetException => record(s"$where ${node.nodeType}.${m.getName}", e.getCause)
            }
          }
        case Some(inner)                       => walk(where, inner, depth)
        case seq: scala.collection.Seq[?]      => seq.foreach(walk(where, _, depth))
        case _                                 => ()
      }

      val replacements: Seq[() => Value] = Seq(
        () => ujson.Null, () => ujson.Num(1), () => ujson.Str("x"), () => ujson.True, () => ujson.Arr(), () => ujson.Obj(),
        () => ujson.Arr(ujson.Null), () => ujson.Arr(ujson.Num(1)), () => ujson.Arr(ujson.Obj("type" -> "FromTheFuture"))
      )

      withProject(fixtures.map(f => f.name.stripSuffix(".fixture") -> f.contentAsString).toMap) { asts =>
        val targets  = asts.toSeq.flatMap { case (name, json) => nodesWithParent(json("ast")).map(name -> _).toSeq }
        val nodeTypes = targets.map(_._2._2("type").str).distinct
        for (_ <- 1 to 6000) {
          val (file, (parentOpt, node)) = targets(random.nextInt(targets.size))
          val keys                      = node.value.keys.toSeq
          val key                       = keys(random.nextInt(keys.size))
          val old                       = node.value.get(key)
          val typeName                  = node("type").str
          val (what, replacement): (String, Option[Value]) = random.nextInt(4) match {
            case 0 => (s"$key=<unknown type>", Some(ujson.Str("FromTheFuture")).filter(_ => key == "type"))
            case 1 => (s"$key=<other type>", Some(ujson.Str(nodeTypes(random.nextInt(nodeTypes.size)))).filter(_ => key == "type"))
            case 2 => (s"$key=<deleted>", None)
            case _ => (s"$key=<wrong value>", Some(replacements(random.nextInt(replacements.size))()))
          }
          replacement match {
            case Some(v) => node.value.put(key, v)
            case None    => node.value.remove(key)
          }
          try {
            val where = s"$file $typeName.$what"
            val root  = parentOpt.getOrElse(node)
            try walk(where, createBabelNode(root), 2)
            catch { case t: Throwable => record(s"$where (decode)", t) }
          } finally old match {
            case Some(v) => node.value.put(key, v)
            case None    => node.value.remove(key)
          }
        }
      }
      withClue(bad.map { case (k, m) => s"$k -> $m" }.mkString("\n", "\n", "\n")) { bad shouldBe empty }
    }
  }
}
