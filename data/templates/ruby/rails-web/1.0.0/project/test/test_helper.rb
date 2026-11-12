ENV["RAILS_ENV"] ||= "test"
require_relative "../config/environment"

# Direct Rails test runs also need a database in a fresh checkout.
ActiveRecord::Base.connection_pool.migration_context.migrate

require "rails/test_help"
