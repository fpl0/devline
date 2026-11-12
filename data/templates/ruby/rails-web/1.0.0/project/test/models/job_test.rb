require "test_helper"

class JobTest < ActiveSupport::TestCase
  test "complete workflow persists an audit trail" do
    job = Job.create!(title: "Brake service", customer: "Alex")
    %w[quote approve start finish collect].each do |action|
      job.transition!(action, expected_version: job.lock_version, estimate_cents: 4500, note: "Step #{action}")
    end
    assert_equal "collected", job.reload.status
    assert_equal 4500, job.estimate_cents
    assert_equal %w[receive quote approve start finish collect], job.job_events.pluck(:action)
    assert_empty job.actions
  end

  test "invalid transitions and stale forms cannot change the job" do
    job = Job.create!(title: "Knife sharpening", customer: "Taylor")
    assert_raises(Job::InvalidTransition) { job.transition!("collect", expected_version: 0) }
    job.transition!("quote", expected_version: 0, estimate_cents: 1200)
    assert_raises(Job::StaleVersion) { job.transition!("approve", expected_version: 0) }
    assert_equal "quoted", job.reload.status
    assert_equal 2, job.job_events.count
  end

  test "quote validation rolls back status and history" do
    job = Job.create!(title: "Service", customer: "Sam")
    assert_raises(Job::InvalidTransition) { job.transition!("quote", expected_version: 0, estimate_cents: -1) }
    assert_equal "received", job.reload.status
    assert_equal 1, job.job_events.count
    assert_not Job.new(title: " ", customer: "Sam").valid?
  end
end
