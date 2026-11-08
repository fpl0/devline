class Note < ApplicationRecord
  validates :body, presence: true, length: { maximum: 1_000 }
end
