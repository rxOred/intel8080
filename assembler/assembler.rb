#!/usr/bin/env ruby

class Assembler8080
  REG_CODES = {
    "B" => 0,
    "C" => 1,
    "D" => 2,
    "E" => 3,
    "H" => 4,
    "L" => 5,
    "M" => 6, # (HL)
    "A" => 7
  }.freeze

  REGPAIR_CODES = {
    "B"  => 0, # BC
    "D"  => 1, # DE
    "H"  => 2, # HL
    "SP" => 3
  }.freeze

  def assemble(source)
    @labels = {}
    @lines  = []

    first_pass(source)
    second_pass
  end

  private

  def first_pass(source)
    addr = 0

    source.each_line.with_index(1) do |line, lineno|
      clean = strip_comment(line)
      next if clean.empty?

      tokens = clean.split(/\s+/)
      next if tokens.empty?

      label = nil

      # label: ...
      if tokens.first.end_with?(":")
        label_name = tokens.shift.chomp(":")
        if @labels.key?(label_name)
          raise "Line #{lineno}: duplicate label '#{label_name}'"
        end
        @labels[label_name] = addr
        # if there is nothing after the label, continue
        if tokens.empty?
          @lines << {
            addr: addr, lineno: lineno, label: label_name,
            mnemonic: nil, operands: [], raw: line
          }
          next
        end
        label = label_name
      end 
      if tokens.first == "ORG"
        puts "Token is ORG... Skipping..."
        if tokens.length != 2
          raise "Line #{lineno}: ORG needs one operand"
        end
        next
        # ignoring the address
      end

      mnemonic = tokens.shift.upcase
      operands = tokens.join(" ")
                         .split(",")
                         .map(&:strip)
                         .reject(&:empty?)

      size = size_of(mnemonic, operands, lineno)

      @lines << {
        addr: addr,
        lineno: lineno,
        label: label,
        mnemonic: mnemonic,
        operands: operands,
        size: size,
        raw: line
      }

      addr += size
    end
  end

  def second_pass
    bytes = []

    @lines.each do |entry|
      mnem = entry[:mnemonic]
      next if mnem.nil? # pure label line

      case mnem
      when "DB"
        entry[:operands].each do |op|
          bytes << (resolve_byte(op, entry) & 0xFF)
        end
      when "DW"
        entry[:operands].each do |op|
          word = resolve_word(op, entry)
          bytes << (word & 0xFF)       # low
          bytes << ((word >> 8) & 0xFF) # high
        end
      when "MOV"
        dest, src = entry[:operands]
        dest_code = REG_CODES[dest.upcase] or asm_error(entry, "Unknown register '#{dest}' in MOV")
        src_code  = REG_CODES[src.upcase]  or asm_error(entry, "Unknown register '#{src}' in MOV")
        opcode = 0x40 + (dest_code << 3) + src_code
        bytes << opcode
      when "MVI"
        reg, imm = entry[:operands]
        reg_code = REG_CODES[reg.upcase] or asm_error(entry, "Unknown register '#{reg}' in MVI")
        opcode = 0x06 + (reg_code << 3)
        val = resolve_byte(imm, entry)
        bytes << opcode
        bytes << (val & 0xFF)
      when "LXI"
        rp, imm = entry[:operands]
        rp_code = REGPAIR_CODES[rp.upcase] or asm_error(entry, "Unknown register pair '#{rp}' in LXI")
        opcode = 0x01 + (rp_code << 4)
        word = resolve_word(imm, entry)
        bytes << opcode
        bytes << (word & 0xFF)        # low
        bytes << ((word >> 8) & 0xFF) # high
      when "INR"
        reg = entry[:operands].first
        reg_code = REG_CODES[reg.upcase] or asm_error(entry, "Unknown register '#{reg}' in INR")
        opcode = 0x04 + (reg_code << 3)
        bytes << opcode
      when "DCR"
        reg = entry[:operands].first
        reg_code = REG_CODES[reg.upcase] or asm_error(entry, "Unknown register '#{reg}' in DCR")
        opcode = 0x05 + (reg_code << 3)
        bytes << opcode  
      when "HLT"
        bytes << 0x76
      else
        asm_error(entry, "Unknown mnemonic '#{mnem}'")
      end
    end

    bytes
  end

  def size_of(mnemonic, operands, lineno)
    case mnemonic
    when "DB"
      raise "Line #{lineno}: DB needs at least 1 operand" if operands.empty?
      operands.length
    when "DW"
      raise "Line #{lineno}: DW needs at least 1 operand" if operands.empty?
      operands.length * 2
    when "MOV"
      1
    when "MVI"
      2
    when "LXI"
      3
    when "HLT"
      1
    when "INR"
      1
    when "DCR"
      1
    else
      raise "Line #{lineno}: unknown mnemonic '#{mnemonic}'"
    end
  end
 
  def resolve_byte(token, entry)
    val = resolve_token(token, entry)
    unless (0..0xFF).include?(val)
      asm_error(entry, "Value #{val} out of 8-bit range")
    end
    val
  end

  def resolve_word(token, entry)
    val = resolve_token(token, entry)
    unless (0..0xFFFF).include?(val)
      asm_error(entry, "Value #{val} out of 16-bit range")
    end
    val
  end

  def resolve_token(token, entry)
    if label_like?(token)
      if @labels.key?(token)
        @labels[token]
      else
        asm_error(entry, "Unknown label '#{token}'")
      end
    else
      parse_number(token, entry)
    end
  end
 
  def strip_comment(line)
    line.split(/[;#]/).first.to_s.strip
  end

  def label_like?(tok)
    !!(tok =~ /\A[A-Za-z_][A-Za-z0-9_$]*\z/)
  end

  def parse_number(tok, entry)
    t = tok.strip.upcase

    if t.start_with?("0X")
      Integer(t[2..], 16)
    elsif t.end_with?("H")
      Integer(t[0..-2], 16)
    elsif t.end_with?("B")
      Integer(t[0..-2], 2)
    else
      Integer(t, 10)
    end
  rescue ArgumentError
    asm_error(entry, "Cannot parse number '#{tok}'")
  end

  def asm_error(entry, msg)
    raise "Line #{entry[:lineno]}: #{msg}\n  >> #{entry[:raw].rstrip}"
  end
end

if __FILE__ == $0
  if ARGV.length != 2
    puts "Usage: ruby asm8080.rb input.asm output.bin"
    exit 1
  end

  input_path  = ARGV[0]
  output_path = ARGV[1]

  source = File.read(input_path)
  asm    = Assembler8080.new
  bytes  = asm.assemble(source)

  File.binwrite(output_path, bytes.pack("C*"))
  puts "Wrote #{bytes.length} bytes to #{output_path}"
end
