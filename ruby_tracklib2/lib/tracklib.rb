require "tracklib/version"
require "rutie"

module TracklibNext
  unless defined?(TrackReader)
    Rutie.new(:ruby_tracklib_next).init 'Init_Tracklib', __dir__
  end
end
