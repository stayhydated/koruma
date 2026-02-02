## AlphanumericValidation

alphanumeric_validation = Must contain only letters and numbers.

## AsciiValidation

ascii_validation = Must contain only ASCII characters.

## Case

case-Camel = camelCase (lower camel case)
case-Kebab = kebab-case
case-Pascal = PascalCase (upper camel case)
case-ShoutyKebab = SCREAMING-KEBAB-CASE
case-ShoutySnake = SCREAMING_SNAKE_CASE
case-Snake = snake_case
case-Title = Title Case
case-Train = Train-Case

## CaseValidation

case_validation = Must be in { $case } case.

## ContainsValidation

contains_validation = Must contain the substring '{ $substring }'.

## CreditCardValidation

credit_card_validation = Not a valid credit card number.

## EmailValidation

email_validation = Not a valid email address.

## IpValidation

ip_validation =
    Not a valid { $kind ->
        [v4] IPv4
        [v6] IPv6
       *[other] IP
    } address.

## LenValidation

len_validation = The length must be between { $min } and { $max }.

## MatchesValidation

matches_validation = Does not match the expected value '{ $other }'.

## NegativeValidation

negative_validation = Must be negative.

## NonEmptyValidation

non_empty_validation = Must not be empty.

## NonNegativeValidation

non_negative_validation = Must be zero or a positive number.

## NonPositiveValidation

non_positive_validation = Must be zero or a negative number.

## PatternValidation

pattern_validation = Does not match the required pattern '{ $pattern }'.

## PhoneNumberValidation

phone_number_validation = Not a valid phone number.

## PositiveValidation

positive_validation = Must be positive.

## PrefixValidation

prefix_validation = Must start with '{ $prefix }'.

## RangeValidation

range_validation = Must be between { $min } and { $max }.

## RequiredValidation

required_validation = This field is required and must not be empty.

## SuffixValidation

suffix_validation = Must end with '{ $suffix }'.

## UrlValidation

url_validation = Not a valid URL.
