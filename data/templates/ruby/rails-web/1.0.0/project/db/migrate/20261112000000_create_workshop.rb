class CreateWorkshop < ActiveRecord::Migration[8.1]
  def change
    create_table :jobs do |t|
      t.string :title, null: false
      t.string :customer, null: false
      t.text :description, null: false, default: ""
      t.string :status, null: false, default: "received"
      t.integer :estimate_cents, null: false, default: 0
      t.integer :lock_version, null: false, default: 0
      t.timestamps
    end
    add_index :jobs, [:status, :created_at]
    add_check_constraint :jobs, "estimate_cents >= 0", name: "positive_estimate"
    create_table :job_events do |t|
      t.references :job, null: false, foreign_key: true
      t.string :action, null: false
      t.string :status, null: false
      t.text :note, null: false, default: ""
      t.timestamps
    end
  end
end
