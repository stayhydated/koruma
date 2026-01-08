## AlphanumericValidation

alphanumeric_validation = 值 '{ $actual }' 必须只包含字母和数字。

## AsciiValidation

ascii_validation = 值 '{ $actual }' 必须只包含 ASCII 字符。

## ContainsValidation

contains_validation = 值 '{ $actual }' 必须包含子串 '{ $substring }'。

## IpKind

ip_kind-Any = 任意 IP 版本
ip_kind-V4 = IPv4
ip_kind-V6 = IPv6

## IpValidation

ip_validation = 值 '{ $actual }' 不是有效的 { $kind } IP 地址。

## LenValidation

len_validation = 长度必须在 { $min } 到 { $max } 之间（实际：{ $actual }）。

## MatchesValidation

matches_validation = 值与期望值 '{ $other }' 不匹配（实际：'{ $actual }'）。

## PrefixValidation

prefix_validation = 值 '{ $actual }' 必须以 '{ $prefix }' 开头。

## RangeValidation

range_validation = 值必须在 { $min } 到 { $max } 之间（实际：{ $actual }）。

## RequiredValidation

required_validation = 此字段为必填，不能为空。

## SuffixValidation

suffix_validation = 值 '{ $actual }' 必须以 '{ $suffix }' 结尾。

## NegativeValidation

negative_validation = 值 '{ $actual }' 必须为负数。

## NonEmptyValidation

non_empty_validation = 该值不能为空。

## NonNegativeValidation

non_negative_validation = 值 '{ $actual }' 必须为零或正数。

## NonPositiveValidation

non_positive_validation = 值 '{ $actual }' 必须为零或负数。

## PositiveValidation

positive_validation = 值 '{ $actual }' 必须为正数。

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

case_validation = 值 '{ $actual }' 必须使用 { $case } 格式。

## CreditCardValidation

credit_card_validation = 值 '{ $actual }' 不是有效的信用卡号。

## EmailValidation

email_validation = 值 '{ $actual }' 不是有效的电子邮件地址。

## PatternValidation

pattern_validation = 值 '{ $actual }' 不匹配所需的模式 '{ $pattern }'。

## PhoneNumberValidation

phone_number_validation = 值 '{ $actual }' 不是有效的电话号码。

## UrlValidation

url_validation = 值 '{ $actual }' 不是有效的 URL。
