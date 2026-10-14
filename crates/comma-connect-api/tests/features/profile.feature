Feature: Authenticated user profile

  Scenario: Fetch the current user profile
    Given an authenticated client
    When I request the current user profile
    Then the response is a JSON object
    And the response contains a string field "id"
