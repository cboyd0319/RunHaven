#!/usr/bin/env bash
# Verifies that the support handoff URL and GitHub issue form stay aligned.
# Fails when required privacy fields, the text upload, or the private-reporting link drift.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

ruby <<'RUBY'
require "yaml"

form_path = ".github/ISSUE_TEMPLATE/runhaven-support.yml"
config_path = ".github/ISSUE_TEMPLATE/config.yml"
abort("missing #{form_path}") unless File.file?(form_path)
abort("missing #{config_path}") unless File.file?(config_path)

form = YAML.safe_load(File.read(form_path), aliases: false, filename: form_path)
abort("support form needs name") unless form["name"].is_a?(String) && !form["name"].empty?
abort("support form needs description") unless form["description"].is_a?(String) && !form["description"].empty?
body = form["body"]
abort("support form needs body fields") unless body.is_a?(Array) && !body.empty?

fields = body.select { |item| item.is_a?(Hash) && item["id"] }
ids = fields.map { |item| item["id"] }
abort("support form field ids must be unique") unless ids.uniq.length == ids.length

problem = fields.find { |item| item["id"] == "what-happened" }
abort("support form needs a required what-happened field") unless
  problem&.dig("type") == "textarea" && problem.dig("validations", "required") == true

upload = fields.find { |item| item["id"] == "support-report" }
abort("support form needs an optional .txt upload") unless
  upload&.dig("type") == "upload" &&
  upload.dig("validations", "required") == false &&
  upload.dig("validations", "accept") == ".txt"

review = fields.find { |item| item["id"] == "privacy-review" }
options = review&.dig("attributes", "options")
abort("support form needs required privacy confirmations") unless
  review&.dig("type") == "checkboxes" &&
  options.is_a?(Array) && options.length >= 2 &&
  options.all? { |option| option["required"] == true }

rust = File.read("crates/runhaven-core/src/support/report/mod.rs")
expected = "https://github.com/cboyd0319/RunHaven/issues/new?template=#{File.basename(form_path)}"
abort("support URL must target #{expected}") unless rust.include?(%Q{"#{expected}"})

config = YAML.safe_load(File.read(config_path), aliases: false, filename: config_path)
abort("blank public issues must stay disabled") unless config["blank_issues_enabled"] == false
security = Array(config["contact_links"]).find { |link| link["name"]&.include?("security") }
abort("template chooser needs private security reporting") unless
  security && security["url"] == "https://github.com/cboyd0319/RunHaven/security/advisories/new"
RUBY
