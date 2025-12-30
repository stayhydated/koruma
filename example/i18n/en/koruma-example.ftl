## NumberRangeValidation

number_range_validation =
    Expected number to be between { $min } and { $max }{ $value ->
        [none] { "" }
       *[other] { ", but got { $value }" }
    }

## StringLengthValidation

string_length_validation =
    Expected text length to be between { $min } and { $max }{ $value ->
        [none] { "" }
       *[other] { ", but got \"{ $value }\"" }
    }
