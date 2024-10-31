#!/usr/bin/env ruby

# Script to update the version in `lib/ruby_ast_gen.rb/version.rb`
require 'fileutils'

VERSION_FILE_PATH = 'lib/ruby_ast_gen.rb/version.rb'


if ARGV.empty?
  puts "Usage: update_version.rb <new_version>"
  exit 1
end

new_version = ARGV[0]
unless new_version.match?(/^v\d+\.\d+\.\d+$/)
  puts "Invalid version format. Please provide a version in the format 'vX.X.X' (e.g., v1.0.0)."
  exit 1
end
stripped_version = new_version.gsub(/^v/, '')

begin
  contents = File.read(VERSION_FILE_PATH)
rescue Errno::ENOENT
  puts "Error: Couldn't find the version file at #{VERSION_FILE_PATH}"
  exit 1
end

new_contents = contents.gsub(/VERSION\s*=\s*["']\d+\.\d+\.\d+["']/, "VERSION = \"#{stripped_version}\"")

begin
  File.write(VERSION_FILE_PATH, new_contents)
  puts "Successfully updated version to #{stripped_version} in #{VERSION_FILE_PATH}"
rescue Errno::EACCES
  puts "Error: Insufficient permissions to update #{VERSION_FILE_PATH}"
  exit 1
end
