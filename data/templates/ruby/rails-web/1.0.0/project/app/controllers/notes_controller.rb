class NotesController < ApplicationController
  def index
    @note = Note.new
    @notes = recent_notes
  end

  def create
    @note = Note.new(params.expect(note: [:body]))

    if @note.save
      redirect_to root_path, status: :see_other
    else
      @notes = recent_notes
      render :index, status: :unprocessable_content
    end
  end

  private

  def recent_notes
    Note.order(created_at: :desc, id: :desc)
  end
end
