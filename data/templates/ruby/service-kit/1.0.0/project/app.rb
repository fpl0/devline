require "dotenv"
require "json"
require "logger"
require "sinatra/base"

class ServiceKit < Sinatra::Base
  set :logger, Logger.new($stdout)

  get "/health" do
    content_type :json
    environment = ENV.fetch("APP_ENVIRONMENT", "local")
    settings.logger.info("health_checked environment=#{environment}")
    JSON.generate(status: "ok", environment:)
  end
end
