defprotocol Mongo.Encoder do
  @fallback_to_any false

  @spec encode(t) :: map()
  def encode(value)
end

## keeps the compiler happy
defimpl Mongo.Encoder, for: String do
  def encode(value) do
    %{string: value}
  end
end
