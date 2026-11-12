class JobsController < ApplicationController
  before_action :find_job, only: [:show, :transition]

  def index
    @status = params[:status].presence
    @query = params[:q].to_s.strip.first(120)
    @jobs = Job.order(created_at: :desc)
    @jobs = @jobs.where(status: @status) if Job::TRANSITIONS.key?(@status)
    if @query.present?
      term = "%#{Job.sanitize_sql_like(@query)}%"
      @jobs = @jobs.where("title LIKE ? OR customer LIKE ?", term, term)
    end
    @jobs = @jobs.limit(100)
    @counts = Job.group(:status).count
    @job = Job.new
  end

  def show; end

  def create
    @job = Job.new(params.require(:job).permit(:title, :customer, :description))
    if @job.save
      redirect_to @job, notice: "Job received. Prepare an estimate when you are ready."
    else
      render :new, status: :unprocessable_entity
    end
  end

  def transition
    version = Integer(params.require(:version), 10)
    amount = params[:estimate_cents].present? ? Integer(params[:estimate_cents], 10) : nil
    @job.transition!(params.require(:event), expected_version: version, estimate_cents: amount, note: params[:note].to_s)
    redirect_to @job, notice: "Job updated.", status: :see_other
  rescue Job::StaleVersion, ActiveRecord::StaleObjectError => error
    @job.reload
    flash.now[:alert] = error.message
    render :show, status: :conflict
  rescue Job::InvalidTransition, ActiveRecord::RecordInvalid, ArgumentError => error
    @job.reload
    flash.now[:alert] = error.message
    render :show, status: :unprocessable_entity
  end

  private

  def find_job
    @job = Job.find(params[:id])
  end
end
