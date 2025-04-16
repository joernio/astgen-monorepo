require 'temple'
require 'erb'

class ErbToRubyTransformer
  def initialize
    @parser = Temple::ERB::Parser.new
    @indent_level = 0
    @current_line = []
    @in_control_block = false
    @control_block_content = []
  end

  def transform(input)
    ast = @parser.call(input)
    content = visit(ast)
    if @in_control_block
      raise ::StandardError, "Invalid ERB Syntax"
    end
    # Wrap everything in a HEREDOC
    <<~RUBY
      <<~HEREDOC
      #{content}
      HEREDOC
    RUBY
  end

  private
  def visit(node)
    return "" unless node.is_a?(Array)

    case node.first
    when :multi
      # Usually the start of an ERB program
      output = []
      node[1..-1].each do |child|
        transformed = visit(child)
        if transformed.strip.empty?
          flush_current_line(output) unless @current_line.empty?
        else
          @current_line << transformed
        end
      end
      flush_current_line(output) unless @current_line.empty?
      output.join("\n")
    when :static
      text = node[1].to_s
      return "" if text.strip.empty?
      if @in_control_block
        # In control blocks, we need to escape newlines and maintain indentation
        escaped_text = text.gsub("\n", "\\n")
        @control_block_content << "#{escaped_text}"
        ""  # Return empty string as we're collecting content
      else
        "#{indent}#{text}"
      end
    when :dynamic
      # Handles <%= %> tags
      code = node[1].to_s.strip
      if @in_control_block
        @control_block_content << "\#{#{code}}"
        ""
      else
        "\#{#{code}}"
      end
    when :escape
      escape_enabled = node[1]
      inner_node = node[2]
      visit(inner_node)
    when :code
      # Handles <% %> tags
      code = node[1].to_s.strip
      if code.start_with?('if', 'unless', 'else', 'elsif', 'end')
        if code.start_with?('if', 'unless')
          @in_control_block = true
          @control_block_content = []
          flush_current_line(@current_line) unless @current_line.empty?
          "\#{#{code}"
        elsif code == 'end'
          @in_control_block = false
          # Join all collected content and wrap in quotes
          content = @control_block_content.join
          @control_block_content = []
          "\"#{content}\"#{code}}"
        else
          # else, elsif
          content = @control_block_content.join
          @control_block_content = []
          "\"#{content}\"#{code}"
        end
      else
        if @in_control_block
          @control_block_content << "#{code}"
          ""
        else
          "\#{#{code}}"
        end
      end
    else
      if node.is_a?(Array) && node.length > 1
        node[1..-1].map { |child| visit(child) }.join
      else
        ""
      end
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
end