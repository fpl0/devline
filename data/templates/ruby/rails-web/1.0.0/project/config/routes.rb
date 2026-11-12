Rails.application.routes.draw do
  get "up" => "rails/health#show", as: :rails_health_check
  root "jobs#index"
  resources :jobs, only: [:index, :show, :create] do
    post :transition, on: :member
  end
end
