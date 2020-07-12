Feature: templates

  @CORR-1 @OPEN
  Scenario: As a builder i should be able to add template
    Given i provide template
    When i provide runtime for evaluation
    Then Template results in value

  @CORR-1 @OPEN
  Scenario: one two three
    Given one
    When two
    Then three
