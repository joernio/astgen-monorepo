require 'temple'
require 'erb'

class ErbToRubyTransformer
  def initialize
    @parser = Temple::ERB::Parser.new
    @in_control_block = false
    @output_tmp_var = "joern__buffer"
    @in_do_block = false
    @inner_buffer = "joern__inner_buffer"
    @current_counter = 0
    @current_lambda_vars = ""
    @output = []
    @static_buff = []
  end

  def transform(input)
    ast = @parser.call(input)
    @output << "#{@output_tmp_var} = \"\""
    visit(ast)
    @output << "return #{@output_tmp_var}"

    if @in_control_block || @in_do_block
      raise ::StandardError, "Invalid ERB Syntax"
    end
    <<~RUBY
      #{@output.join("\n")}
    RUBY
  end

  private
  def visit(node)
    return "" unless node.is_a?(Array)
    case node.first
    when :multi
      node[1..-1].each do |child|
        visit(child)
      end
    when :static
      unless node[1].to_s != nil && node[1].to_s.strip.empty?
        @static_buff << "\"#{node[1].to_s.gsub('"', '\"').strip}\""
      end
    when :dynamic
      unless node[1].to_s != nil && node[1].to_s.strip.empty?
        @output << "\"#{node[1].to_s.gsub('"', '\"')}\""
      end
    when :escape
      unless @static_buff.empty?
        buffer_to_use = if @in_do_block then "#{@inner_buffer}" else "#{@output_tmp_var}" end
        @output << "#{buffer_to_use} << \"#{@static_buff.join('\n').gsub(/(?<!\\)"/, '')}\""
        @static_buff = [] # clear static buffer
      end

      escape_enabled = node[1]
      inner_node = node[2]
      code = inner_node[1].to_s

      # Do block with variable found, lower
      if is_do_block(code)
        lower_do_block(code)
      elsif @in_do_block
        template_call = if escape_enabled then "joern__template_out_raw" else "joern__template_out_escape" end
        @output << "#{@inner_buffer} << #{template_call}(#{code})"
      else
        template_call = if escape_enabled then "joern__template_out_raw" else "joern__template_out_escape" end
        @output << "#{@output_tmp_var} << #{template_call}(#{code})"
      end
    when :code
      unless @static_buff.empty?
        buffer_to_use = if @in_do_block then "#{@inner_buffer}" else "buffer" end
        @output << "#{buffer_to_use} << \"#{@static_buff.join('\n').gsub(/(?<!\\)"/, '')}\""
        @static_buff = [] # clear static buffer
      end

      stripped_code = node[1].to_s.strip
      code = node[1].to_s
      # Using this to determine if we should throw a StandardError for "invalid" ERB
      if is_control_struct_start(stripped_code)
        @in_control_block = true
        @output << stripped_code
      elsif code.start_with?("end")
        if @in_do_block
          @in_do_block = false
          @output << "#{@inner_buffer}"
          @output << "end"
          @output << "#{@output_tmp_var} << #{current_lambda}.call(#{@current_lambda_vars})"
        else
          @in_control_block = false
          @output << "end"
        end
      else
        if is_do_block(code)
          lower_do_block(code)
        end
      end
    when :newline
    else
      RubyAstGen::Logger::debug("Invalid node type: #{node}")
    end
  end

  def is_control_struct_start(line)
    line.start_with?('if', 'unless', 'elsif', 'else', /@?.+\.(each|each_with_index)\sdo/)
  end

  def lambda_incrementor()
    new_lambda = "rails_lambda_#{@current_counter}"
    @current_counter += 1
    new_lambda
  end

  def current_lambda()
    "rails_lambda_#{@current_counter-1}"
  end

  def lower_do_block(code)
    if (code_match = code.match(/do\s+(?:\|([^|]*)\|)?/))
      @current_lambda_vars = code_match[1]
      before_do, _ = code.split(/\bdo\b/)
      unless before_do.nil?
        method_call = before_do.strip
        call_name, rest = method_call.split(' ', 2)
        if rest != nil && !rest.start_with?('(') && !rest.end_with?(')')
          method_call = "#{call_name}(#{rest})"
        end
        @output << "#{@output_tmp_var} << #{method_call}"
      end
      @in_do_block = true
      @output << "#{lambda_incrementor} = lambda do |#{@current_lambda_vars}|"
      @output << "#{@inner_buffer} = \"\""
    end
  end

  def is_do_block(code)
    code.match(/do\s*(?:\|([^|]*)\|)?/)
  end
end




