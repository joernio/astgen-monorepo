require 'temple'
require 'erb'

class ErbToRubyTransformer
  def initialize
    @parser = Temple::ERB::Parser.new
    @indent_level = 0
    @current_line = []
    @in_control_block = false
    @output_tmp_var = "tmp0"
    @is_first_output = true
    @output = []
    @no_control_struct = true
    @open_heredoc = false
  end

  def transform(input)
    ast = @parser.call(input)
    content = "#{@output_tmp_var} = \"\" \n#{visit(ast)}"
    if @in_control_block
      raise ::StandardError, "Invalid ERB Syntax"
    end
    <<~RUBY
      #{content}
      return #{@output_tmp_var}
    RUBY
  end

  private
  def visit(node)
    return "" unless node.is_a?(Array)
    case node.first
    when :multi
      node[1..-1].each do |child|
        transformed = visit(child)
        unless transformed.strip.empty?
          if @is_first_output
            @open_heredoc = true
            @current_line << "#{@output_tmp_var} += <<-HEREDOC\n"
            @is_first_output = false
          end
          @current_line << transformed
        end
      end

      if @open_heredoc
        @current_line << "\nHEREDOC\n"
        @open_heredoc = false
      end

      flush_current_line(@output) unless @current_line.empty?
      @output.join("\n")
    when :static
      "#{node[1].to_s}"
    when :dynamic
      "#{node[1].to_s}"
    when :escape
      escape_enabled = node[1]
      inner_node = node[2]
      code = inner_node[1].to_s.strip
      template_call = if escape_enabled then "joern__template_out_raw" else "joern__template_out_escape" end
      "\#{#{template_call}(#{code})}"
    when :code
      code = node[1].to_s.strip
      # Using this to determine if we should throw a StandardError for "invalid" ERB
      if is_control_struct_start(code)
        @in_control_block = true
      elsif code.start_with?("end")
        @in_control_block = false
      end

      if @open_heredoc
        @open_heredoc = false
        @current_line << "\nHEREDOC"
      end

      @current_line << "\n#{node[1].to_s.strip}\n"
      @is_first_output = true
      ""
    when :newline
      ""
    else
      RubyAstGen::Logger::debug("Invalid node type: #{node}")
      ""
    end
  end

  def indent
    "  " * @indent_level
  end

  def flush_current_line(output)
    unless @current_line.empty?
      line = @current_line.join.rstrip
      output << line unless line.empty?
      @current_line.clear
    end
  end

  def is_control_struct_start(line)
    line.start_with?('if', 'unless', 'elsif', 'else', /@?\w+\.each\sdo/)
  end

  def is_control_struct(line)
    is_control_struct_start(line) || line.start_with?('end')
  end
end




