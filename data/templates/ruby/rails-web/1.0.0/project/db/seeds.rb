Job.find_or_create_by!(title: "City bicycle service", customer: "Alex") { |job| job.description = "Inspect brakes and replace the rear cable." }
Job.find_or_create_by!(title: "Kitchen knife sharpening", customer: "Taylor") { |job| job.description = "Three chef knives; collection on Friday." }
