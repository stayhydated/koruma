## AlphanumericValidation

alphanumeric_validation = La valeur « { $actual } » doit contenir uniquement des lettres et des chiffres.

## AsciiValidation

ascii_validation = La valeur « { $actual } » doit contenir uniquement des caractères ASCII.

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

case_validation = La valeur « { $actual } » doit être en casse { $case }.

## ContainsValidation

contains_validation = La valeur « { $actual } » doit contenir la sous-chaîne « { $substring } ».

## CreditCardValidation

credit_card_validation = La valeur « { $actual } » n'est pas un numéro de carte de crédit valide.

## EmailValidation

email_validation = La valeur « { $actual } » n'est pas une adresse e-mail valide.

## IpValidation

ip_validation =
    Ce n'est pas une adresse { $kind ->
        [v4] IPv4
        [v6] IPv6
       *[other] IP
    } valide.

## LenValidation

len_validation = La longueur doit être comprise entre { $min } et { $max } (actuelle : { $actual }).

## MatchesValidation

matches_validation = La valeur ne correspond pas à la valeur attendue « { $other } » (actuelle : « { $actual } »).

## NegativeValidation

negative_validation = La valeur « { $actual } » doit être négative.

## NonEmptyValidation

non_empty_validation = La valeur ne doit pas être vide.

## NonNegativeValidation

non_negative_validation = La valeur « { $actual } » doit être nulle ou positive.

## NonPositiveValidation

non_positive_validation = La valeur « { $actual } » doit être nulle ou négative.

## PatternValidation

pattern_validation = La valeur « { $actual } » ne correspond pas au motif requis « { $pattern } ».

## PhoneNumberValidation

phone_number_validation = La valeur « { $actual } » n'est pas un numéro de téléphone valide.

## PositiveValidation

positive_validation = La valeur « { $actual } » doit être positive.

## PrefixValidation

prefix_validation = La valeur « { $actual } » doit commencer par « { $prefix } ».

## RangeValidation

range_validation = La valeur doit être comprise entre { $min } et { $max } (actuelle : { $actual }).

## RequiredValidation

required_validation = Ce champ est obligatoire et ne doit pas être vide.

## SuffixValidation

suffix_validation = La valeur « { $actual } » doit se terminer par « { $suffix } ».

## UrlValidation

url_validation = La valeur « { $actual } » n'est pas une URL valide.
