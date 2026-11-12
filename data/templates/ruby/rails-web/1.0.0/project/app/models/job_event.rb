class JobEvent < ApplicationRecord
  belongs_to :job
  validates :action, :status, presence: true
end
