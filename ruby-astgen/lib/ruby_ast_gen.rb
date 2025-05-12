require 'parser/current'
require 'fileutils'
require 'json'
require 'thread'

require_relative 'ruby_ast_gen/version'
require_relative 'ruby_ast_gen/node_handling'
require_relative 'ruby_ast_gen/erb_to_ruby_transformer'

module RubyAstGen

  module Logger

    def self.debug(message)
      puts "[DEBUG] #{message}"
    end

    def self.info(message)
      puts "[INFO] #{message}"
    end

    def self.warn(message)
      puts "[WARN] #{message}"
    end

    def self.error(message)
      puts "[ERR] #{message}"
    end
  end

  # Main method to parse the input and generate the AST output
  def self.parse(opts)
    if opts[:debug]
      RubyAstGen::Logger::debug "CLI Arguments received: #{opts}"
    end

    input_path = opts[:input]
    output_dir = opts[:output]
    exclude_regex = Regexp.new(opts[:exclude])

    if opts[:debug]
      RubyAstGen::Logger::debug "Exclude Regex Received: #{exclude_regex}"
    end

    FileUtils.mkdir_p(output_dir)

    if File.file?(input_path)
      process_file(input_path, output_dir, exclude_regex, input_path)
    elsif File.directory?(input_path)
      process_directory(input_path, output_dir, exclude_regex)
    else
      RubyAstGen::Logger::error "#{input_path} is neither a file nor a directory."
      exit 1
    end
  end

  private

  # Process a single file and generate its AST
  def self.process_file(file_path, output_dir, exclude_regex, base_dir)
    # Get the relative path of the file to apply exclusion rules
    relative_path = file_path.sub(%r{^.*\/}, '')
    relative_input_path = file_path.sub("#{base_dir}/", '')
    # Skip if the file matches the exclusion regex
    if exclude_regex && exclude_regex.match?(relative_input_path)
      RubyAstGen::Logger::info "Excluding: #{relative_input_path}"
      return
    end

    return unless ruby_file?(file_path) || erb_file?(file_path) # Skip if it's not a Ruby-related file

    begin
      ast = parse_file(file_path, relative_input_path)
      return unless ast

      output_path = File.join(output_dir, "#{relative_path}.json")

      File.write(output_path, JSON.pretty_generate(ast))
      RubyAstGen::Logger::info "Processed: #{relative_input_path} -> #{output_path}"
    rescue StandardError => e
      RubyAstGen::Logger::error "'#{relative_input_path}' - #{e.message}"
    end
  end

  def self.process_directory(dir_path, output_dir, exclude_regex, max_threads = 10)
    threads = []
    queue = Queue.new

    Dir.glob("#{dir_path}/**/*").each do |path|
      next unless File.file?(path) && (ruby_file?(path) || erb_file?(path))
      relative_dir = path.sub("#{dir_path}/", '')
      next if exclude_regex.match?(relative_dir)

      queue << path
    end

    max_threads.times do
      threads << Thread.new do
        until queue.empty?
          begin
            path = queue.pop(true) rescue nil # Non-blocking pop
            next unless path

            relative_path = path.sub(dir_path, '')
            output_subdir = File.join(output_dir, File.dirname(relative_path))
            FileUtils.mkdir_p(output_subdir)

            process_file(path, output_subdir, exclude_regex, dir_path)
          rescue => e
            RubyAstGen::Logger::error "Error processing #{path}: #{e.message}"
          end
        end
      end
    end

    threads.each(&:join)
  end

  def self.parse_file(file_path, relative_input_path)
    code =
      if ruby_file?(file_path)
        File.read(file_path)
      else
        file_content = File.read(file_path)
        get_erb_content(file_content)
      end
    buffer = Parser::Source::Buffer.new(file_path)
    buffer.source = code
    parser = Parser::CurrentRuby.new
    ast = parser.parse(buffer)
    return unless ast
    json_ast = NodeHandling::ast_to_json(ast, code, file_path: relative_input_path)
    json_ast[:file_path] = file_path
    json_ast[:rel_file_path] = relative_input_path
    json_ast
  rescue Parser::SyntaxError => e
    RubyAstGen::Logger::error "Failed to parse #{file_path}: #{e.message}"
    nil
  end

  def self.ruby_file?(file_path)
    ext = File.extname(file_path)
    ['.rb', '.gemspec', 'Rakefile'].include?(ext) || file_path.end_with?('.rb')
  end

  def self.erb_file?(file_path)
    ext = File.extname(file_path)
    ['.erb'].include?(ext) || file_path.end_with?('.erb')
  end

  def self.get_erb_content(file_content)
    begin
      transformer = ErbToRubyTransformer.new
      transformer.transform(file_content)
    rescue StandardError => e
      # Wrap the file_content in HEREDOC so the AST parser gives a String output of the ERB file
      # in case the transformation fell over
      <<~RUBY
        <<~HEREDOC
          #{file_content}
        HEREDOC
      RUBY
    end
  end
end
