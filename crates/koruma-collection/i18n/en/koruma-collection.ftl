## AlphanumericValidation

alphanumeric_validation = Alphanumeric Validation { $actual }

## AsciiValidation

ascii_validation = Ascii Validation { $actual }

## ContainsValidation

contains_validation = Contains Validation { $substring } { $actual }

## IpKind

ip_kind-Any = Any
ip_kind-V4 = V4
ip_kind-V6 = V6

## IpValidation

ip_validation = Ip Validation { $kind } { $actual }

## LenValidation

len_validation = Len Validation { $min } { $max } { $actual }

## MatchesValidation

matches_validation = Matches Validation { $other } { $actual }

## PrefixValidation

prefix_validation = Prefix Validation { $prefix } { $actual }

## RangeValidation

range_validation = Range Validation { $min } { $max } { $actual }

## RequiredValidation

required_validation = Required Validation

## SuffixValidation

suffix_validation = Suffix Validation { $suffix } { $actual }