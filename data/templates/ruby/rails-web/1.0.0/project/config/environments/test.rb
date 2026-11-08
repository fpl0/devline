require "tmpdir"

Rails.application.configure do
  config.enable_reloading = false
  config.eager_load = false
  config.consider_all_requests_local = true
  config.action_controller.allow_forgery_protection = false
  config.active_record.dump_schema_after_migration = false

  test_dir = ENV["DEVLINE_RAILS_TEST_DIR"]
  unless test_dir
    test_dir = Dir.mktmpdir("devline-rails-test.")
    owner = Process.pid
    at_exit { FileUtils.remove_entry(test_dir) if Process.pid == owner }
    # Rails may invoke a separate task to load the test schema.
    ENV["DEVLINE_RAILS_TEST_DIR"] = test_dir
  end

  config.paths["log"] = [File.join(test_dir, "test.log")]
  config.secret_key_base = SecureRandom.hex(64)
end
