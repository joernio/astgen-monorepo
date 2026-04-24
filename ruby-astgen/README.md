# Ruby AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for Ruby source files. Uses the [`parser` gem](https://github.com/whitequark/parser), which supports Ruby 1.8 through 3.2 syntax with backwards-compatible AST formats.

## Supported languages

| Language | Tool used    | Notes                     |
| -------- | ------------ | ------------------------- |
| Ruby     | `parser` gem | Ruby 1.8 – 3.2 syntax     |

## Building

The release uses JRuby to enable a standalone distribution. With JRuby installed:

```bash
jruby -S bundle install
```

Alternatively, using the JRuby complete JAR:

```bash
curl 'https://repo1.maven.org/maven2/org/jruby/jruby-complete/9.4.8.0/jruby-complete-9.4.8.0.jar' \
    --output jruby.jar
java -jar jruby.jar -S gem install bundler --install-dir vendor/bundle/jruby/3.1.0
java -jar jruby.jar -s vendor/bundle/jruby/3.1.0/bin/bundle install
```

> If switching between native `ruby` and `jruby` (or vice versa), delete the `vendor/` folder before running `jruby -S bundle install` to avoid a clash.

## Testing

```bash
rake spec
```

You can also run `bin/console` for an interactive prompt.

## Usage

```
Usage:
  -i, --input      The input file or directory (required)
  -o, --output     The output directory (default: '.ast')
  -e, --exclude    The exclusion regex (default: '^(tests?|vendor|spec)')
  -d, --debug      Enable debug logging
      --version    Print the version
      --help       Print usage
```

## Example

```bash
jruby -S bundle exec exe/ruby_ast_gen -i <path to project> -o <path to output>
```

Using the JRuby complete JAR:

```bash
java -jar jruby.jar -S vendor/bundle/jruby/3.1.0/bin/bundle exec exe/ruby_ast_gen -i <input> -o <output>
```

## License

Available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
