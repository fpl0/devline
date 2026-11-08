require "test_helper"

class NotesTest < ActionDispatch::IntegrationTest
  test "a saved note appears on the home page as escaped text" do
    note = "<script>My offline note</script>"
    get root_path
    assert_response :success
    assert_select "form[action=?]", notes_path
    assert_select "link[rel=stylesheet][href^='/assets/application']"

    assert_difference("Note.count", 1) do
      post notes_path, params: { note: { body: note } }
    end
    assert_response :see_other
    follow_redirect!
    assert_response :success
    assert_select ".notes li", text: note
    assert_select "script", count: 0
  end

  test "blank notes are rejected" do
    assert_no_difference("Note.count") do
      post notes_path, params: { note: { body: " " } }
    end
    assert_response :unprocessable_content
    assert_select "#note-errors", text: /Body can't be blank/
  end

  test "the health endpoint responds" do
    get rails_health_check_path
    assert_response :success
  end
end
