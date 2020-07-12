Feature: templates

  @CORR-1 @OPEN
  Scenario: As a builder i should be able to add template
    Given i provide template
    When i provide runtime for evaluation
    Then Template results in value

  @CORR-1 @CORR-2 @OPEN
  Scenario: one two three
    Given one
    When two
    Then three

  @CORR-2 @OPEN
  Scenario Outline: Something
    Given he is "<nature>"
    When he drives
    Then he will be hit if "<nature>" is "calm"
    Examples:
      |nature     |
      | calm      |
      | passionate|
