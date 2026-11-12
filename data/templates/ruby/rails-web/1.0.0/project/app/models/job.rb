class Job < ApplicationRecord
  class InvalidTransition < StandardError; end
  class StaleVersion < StandardError; end

  TRANSITIONS = {
    "received" => { "quote" => "quoted", "cancel" => "cancelled" },
    "quoted" => { "quote" => "quoted", "approve" => "approved", "cancel" => "cancelled" },
    "approved" => { "start" => "in_progress", "cancel" => "cancelled" },
    "in_progress" => { "finish" => "ready" },
    "ready" => { "collect" => "collected" },
    "collected" => {}, "cancelled" => {},
  }.freeze

  has_many :job_events, -> { order(:id) }, dependent: :destroy
  validates :title, :customer, presence: true, length: { maximum: 120 }
  validates :description, length: { maximum: 5000 }
  validates :status, inclusion: { in: TRANSITIONS.keys }
  validates :estimate_cents, numericality: { only_integer: true, greater_than_or_equal_to: 0, less_than_or_equal_to: 100_000_000 }
  before_validation { self.title = title.to_s.strip; self.customer = customer.to_s.strip }
  after_create { job_events.create!(action: "receive", status: status, note: "Job received") }

  def actions
    TRANSITIONS.fetch(status).keys
  end

  def transition!(action, expected_version:, estimate_cents: nil, note: "")
    with_lock do
      raise StaleVersion, "This job changed. Review the latest details before trying again." unless lock_version == expected_version
      target = TRANSITIONS.fetch(status)[action]
      raise InvalidTransition, "#{action.humanize} is not available for a #{status.humanize.downcase} job." unless target
      raise InvalidTransition, "Notes must be at most 1000 characters." if note.length > 1000
      attributes = { status: target }
      if action == "quote"
        raise InvalidTransition, "Enter a positive whole number of cents." unless estimate_cents.is_a?(Integer) && estimate_cents.positive?
        attributes[:estimate_cents] = estimate_cents
      end
      update!(attributes)
      job_events.create!(action: action, status: target, note: note.strip)
    end
    self
  end
end
