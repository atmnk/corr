Feature: templates

  @CORR-1 @OPEN
  Scenario: As a builder i should be able to add template
    Given i provide template
    When i provide runtime for evaluation
    Then Template results in value
