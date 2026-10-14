//! Cucumber tests for the comma connect API client.

use cucumber::{given, then, when, World as _};
use comma_connect_api::Client;
use comma_test_utils::{load_test_env, require_dongle, require_token, World};

#[given("an authenticated client")]
async fn authenticated_client(world: &mut World) -> comma_test_utils::Result<()> {
    load_test_env();
    let token = require_token()?;
    world.client = Some(Client::new(token));
    Ok(())
}

#[given("the test dongle ID is available")]
async fn dongle_available(_world: &mut World) -> comma_test_utils::Result<()> {
    load_test_env();
    let _ = require_dongle()?;
    Ok(())
}

#[when("I request the current user profile")]
async fn request_profile(world: &mut World) -> comma_test_utils::Result<()> {
    let client = world.client.as_ref().unwrap();
    let profile = client.get_me().await?;
    world.last_json = Some(serde_json::to_value(&profile)?);
    Ok(())
}

#[when("I request the list of devices")]
async fn request_devices(world: &mut World) -> comma_test_utils::Result<()> {
    let client = world.client.as_ref().unwrap();
    let devices = client.get_devices().await?;
    world.last_json = Some(serde_json::to_value(&devices)?);
    Ok(())
}

#[when("I request the most recent routes for the dongle")]
async fn request_routes(world: &mut World) -> comma_test_utils::Result<()> {
    let client = world.client.as_ref().unwrap();
    let dongle = require_dongle()?;
    let routes = client.get_routes(&dongle, Some(5), None).await?;
    world.last_json = Some(serde_json::to_value(&routes)?);
    Ok(())
}

#[when("I request the uploaded files for the most recent route of the dongle")]
async fn request_route_files(world: &mut World) -> comma_test_utils::Result<()> {
    let client = world.client.as_ref().unwrap();
    let dongle = require_dongle()?;
    let routes = client.get_routes(&dongle, Some(1), None).await?;
    let route = routes
        .first()
        .ok_or("no routes returned for the dongle")?;
    let files = client.get_route_files(&route.fullname).await?;
    world.last_json = Some(serde_json::to_value(&files)?);
    world.last_files = Some(files);
    Ok(())
}

#[when("I request the upload queue for the dongle")]
async fn request_upload_queue(world: &mut World) -> comma_test_utils::Result<()> {
    let client = world.client.as_ref().unwrap();
    let dongle = require_dongle()?;
    let queue = client.list_upload_queue(&dongle).await?;
    world.last_json = Some(serde_json::to_value(&queue)?);
    Ok(())
}

#[when("I request an upload URL for the first segment of the most recent route")]
async fn request_upload_url(world: &mut World) -> comma_test_utils::Result<()> {
    let client = world.client.as_ref().unwrap();
    let dongle = require_dongle()?;
    let routes = client.get_routes(&dongle, Some(1), None).await?;
    let route = routes.first().ok_or("no routes returned")?;
    let ts = route
        .fullname
        .split('|')
        .nth(1)
        .ok_or("route name missing timestamp")?;
    let path = format!("{}--0/fcamera.hevc", ts);
    let urls = client
        .get_upload_urls(&dongle, &[path], Some(1))
        .await?;
    world.last_json = Some(serde_json::to_value(&urls)?);
    Ok(())
}

#[then("the first element of the response has a string field \"url\"")]
async fn first_element_has_url(world: &mut World) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    let arr = v.as_array().ok_or("response is not an array")?;
    let first = arr.first().ok_or("array is empty")?;
    let obj = first.as_object().ok_or("first element is not an object")?;
    let url = obj.get("url").ok_or("missing url")?;
    assert!(url.is_string(), "url is not a string");
    Ok(())
}

#[then("the response is a JSON object")]
async fn response_is_object(world: &mut World) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    assert!(v.is_object(), "expected object, got {v}");
    Ok(())
}

#[then("the response is a JSON array")]
async fn response_is_array(world: &mut World) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    assert!(v.is_array(), "expected array, got {v}");
    Ok(())
}

#[then(regex = r#"the response contains a string field \"(?P<field>.*)\""#)]
async fn response_has_string_field(world: &mut World, field: String) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    let obj = v.as_object().ok_or("response is not an object")?;
    let val = obj.get(&field).ok_or(format!("missing field {field}"))?;
    assert!(val.is_string(), "field {field} is not a string: {val}");
    Ok(())
}

#[then(regex = r#"the response contains an array field \"(?P<field>.*)\""#)]
async fn response_has_array_field(world: &mut World, field: String) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    let obj = v.as_object().ok_or("response is not an object")?;
    let val = obj.get(&field).ok_or(format!("missing field {field}"))?;
    assert!(val.is_array(), "field {field} is not an array: {val}");
    Ok(())
}

#[then(regex = r#"every element of the response is an object with a string field \"(?P<field>.*)\""#)]
async fn every_element_has_string_field(world: &mut World, field: String) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    let arr = v.as_array().ok_or("response is not an array")?;
    for (i, el) in arr.iter().enumerate() {
        let obj = el.as_object().ok_or(format!("element {i} is not an object"))?;
        let val = obj.get(&field).ok_or(format!("element {i} missing {field}"))?;
        assert!(val.is_string(), "element {i} field {field} is not a string");
    }
    Ok(())
}

#[then(regex = r#"if the array is non-empty, every element has a string field \"(?P<field>.*)\""#)]
async fn nonempty_elements_have_string_field(world: &mut World, field: String) -> comma_test_utils::Result<()> {
    let v = world.last_json.as_ref().ok_or("no response stored")?;
    let arr = v.as_array().ok_or("response is not an array")?;
    if arr.is_empty() {
        return Ok(());
    }
    for (i, el) in arr.iter().enumerate() {
        let obj = el.as_object().ok_or(format!("element {i} is not an object"))?;
        let val = obj.get(&field).ok_or(format!("element {i} missing {field}"))?;
        assert!(val.is_string(), "element {i} field {field} is not a string");
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    load_test_env();
    World::cucumber()
        .run("tests/features")
        .await;
}
