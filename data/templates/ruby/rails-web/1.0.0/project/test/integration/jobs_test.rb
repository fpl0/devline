require "test_helper"

class JobsTest < ActionDispatch::IntegrationTest
  test "receive search quote and reject a stale browser form" do
    get root_path
    assert_response :success
    assert_select "h1", "Workshop Desk"
    post jobs_path, params: { job: { title: "Brake service", customer: "Alex", description: "Front brake" } }
    job = Job.last
    assert_redirected_to job_path(job)
    follow_redirect!
    assert_select "h1", "Brake service"
    post transition_job_path(job), params: { event: "quote", version: "0", estimate_cents: "4500" }
    assert_redirected_to job_path(job)
    post transition_job_path(job), params: { event: "approve", version: "0" }
    assert_response :conflict
    get jobs_path, params: { q: "Alex", status: "quoted" }
    assert_select "h3 a", "Brake service"
  end

  test "invalid intake is shown without creating data and user text is escaped" do
    assert_no_difference("Job.count") { post jobs_path, params: { job: { title: " ", customer: "Alex" } } }
    assert_response :unprocessable_entity
    job = Job.create!(title: "<script>bad</script>", customer: "Sam")
    get job_path(job)
    assert_response :success
    assert_includes response.body, "&lt;script&gt;bad&lt;/script&gt;"
  end
end
