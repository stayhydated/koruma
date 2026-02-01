## AlphanumericValidation

alphanumeric_validation = Doit contenir uniquement des lettres et des chiffres.

## AsciiValidation

ascii_validation = Doit contenir uniquement des caractères ASCII.

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

case_validation = Doit être en casse { $case }.

## ContainsValidation

contains_validation = Doit contenir la sous-chaîne « { $substring } ».

## CreditCardValidation

credit_card_validation = N'est pas un numéro de carte de crédit valide.

## EmailValidation

email_validation = N'est pas une adresse e-mail valide.

## IpValidation

ip_validation =
    Ce n'est pas une adresse { $kind ->
        [v4] IPv4
        [v6] IPv6
       *[other] IP
    } valide.

## LenValidation

len_validation = La longueur doit être comprise entre { $min } et { $max }.

## MatchesValidation

matches_validation = Ne correspond pas à la valeur attendue « { $other } ».

## NegativeValidation

negative_validation = Doit être négatif.

## NonEmptyValidation

non_empty_validation = Ne doit pas être vide.

## NonNegativeValidation

non_negative_validation = Doit être nul ou positif.

## NonPositiveValidation

non_positive_validation = Doit être nul ou négatif.

## PatternValidation

pattern_validation = Ne correspond pas au motif requis « { $pattern } ».

## PhoneNumberValidation

phone_number_validation = N'est pas un numéro de téléphone valide.

## PositiveValidation

positive_validation = Doit être positif.

## PrefixValidation

prefix_validation = Doit commencer par « { $prefix } ».

## RangeValidation

range_validation = Doit être compris entre { $min } et { $max }.

## RequiredValidation

required_validation = Ce champ est obligatoire et ne doit pas être vide.

## SuffixValidation

suffix_validation = Doit se terminer par « { $suffix } ».

## UrlValidation

url_validation = N'est pas une URL valide.
