// String builders for .NET Φ marker lines.

pub fn build_controller_line(class_name: &str, route: Option<&str>) -> String {
    match route {
        Some(route) => format!("Φctrl:{class_name} [{route}]"),
        None => format!("Φctrl:{class_name}"),
    }
}

pub fn build_api_controller_line(class_name: &str) -> String {
    format!("Φapi:{class_name}")
}

pub fn build_action_line(
    verb: &str,
    name: &str,
    params: &str,
    return_type: Option<&str>,
) -> String {
    match return_type {
        Some(return_type) => format!("Φaction:{verb} {name}({params}) → {return_type}"),
        None => format!("Φaction:{verb} {name}({params})"),
    }
}

pub fn build_model_line(model_name: &str) -> String {
    format!("Φmodel:{model_name}")
}

pub fn build_auth_line(policy: Option<&str>) -> String {
    match policy {
        Some(policy) => format!("Φauth:{policy}"),
        None => "Φauth:true".to_string(),
    }
}

pub fn build_ef_line(class_name: &str) -> String {
    format!("Φef:{class_name}")
}

pub fn build_dbset_line(name: &str) -> String {
    format!("Φdbset:{name}")
}

pub fn build_entity_line(name: &str, fields: &[String]) -> String {
    if fields.is_empty() {
        format!("Φentity:{name}")
    } else {
        format!("Φentity:{name} {{ {} }}", fields.join(", "))
    }
}

#[allow(dead_code)]
pub fn build_relationship_line(name: &str, target: &str) -> String {
    format!("Φrel:{name} → {target}")
}

pub fn build_config_line(class_name: &str) -> String {
    format!("Φcfg:{class_name}")
}

pub fn build_mapper_line(class_name: &str) -> String {
    format!("Φmap:{class_name}")
}

pub fn build_mapfrom_line(source: &str, destination: &str) -> String {
    format!("Φmapfrom:{source} → {destination}")
}

pub fn build_ignore_line(member: &str) -> String {
    format!("Φignore:{member}")
}

pub fn build_projection_line(target: &str) -> String {
    format!("Φproj:{target}")
}

pub fn build_hub_line(class_name: &str, client_interface: Option<&str>) -> String {
    match client_interface {
        Some(client_interface) => format!("Φhub:{class_name} [{client_interface}]"),
        None => format!("Φhub:{class_name}"),
    }
}

pub fn build_hub_method_line(name: &str, params: &str, target: &str) -> String {
    format!("Φmethod:{name}({params}) → {target}")
}

#[allow(dead_code)]
pub fn build_client_line(interface: &str, method: &str, params: &str) -> String {
    format!("Φclient:{interface}.{method}({params})")
}

pub fn build_group_line(group_name: &str) -> String {
    format!("Φgroup:{group_name}")
}

pub fn build_user_line(user_id: &str) -> String {
    format!("Φuser:{user_id}")
}

pub fn build_stream_line(method_name: &str, stream_type: &str) -> String {
    format!("Φstream:{method_name} → {stream_type}")
}

pub fn build_connection_line(event: &str) -> String {
    format!("Φconn:{event}")
}

pub fn build_json_line(config: &str) -> String {
    format!("Φjson:{config}")
}

pub fn build_property_line(name: &str) -> String {
    format!("Φprop:{name}")
}

pub fn build_service_line(class_name: &str) -> String {
    format!("Φsvc:{class_name}")
}

pub fn build_di_line(service: &str, registration: &str) -> String {
    format!("Φdi:{service} → {registration}")
}

pub fn build_common_line(attribute: &str) -> String {
    format!("Φcommon:{attribute}")
}

pub fn build_validator_line(class_name: &str) -> String {
    format!("Φvalid:{class_name}")
}

pub fn build_rule_line(property: &str, rules: &[String]) -> String {
    if rules.is_empty() {
        format!("Φrule:{property}")
    } else {
        format!("Φrule:{property} → {}", rules.join(", "))
    }
}

pub fn build_custom_validator_line(name: &str) -> String {
    format!("Φcustom:{name}")
}

pub fn build_identity_line(class_name: &str) -> String {
    format!("Φidentity:{class_name}")
}

pub fn build_jwt_line(config: &str) -> String {
    format!("Φjwt:{config}")
}

pub fn build_cache_line(cache_type: &str) -> String {
    format!("Φcache:{cache_type}")
}

pub fn build_output_line(config: &str) -> String {
    format!("Φoutput:{config}")
}

pub fn build_job_line(name: &str) -> String {
    format!("Φjob:{name}")
}

pub fn build_log_line(pattern: &str) -> String {
    format!("Φlog:{pattern}")
}

pub fn build_metric_line(provider: &str) -> String {
    format!("Φmetric:{provider}")
}

pub fn build_test_class_line(class_name: &str) -> String {
    format!("Φtestcls:{class_name}")
}

pub fn build_test_line(method_name: &str, rows: usize) -> String {
    if rows == 0 {
        format!("Φtest:{method_name}")
    } else {
        format!("Φtest:{method_name} [rows={rows}]")
    }
}

pub fn build_mock_line(dependency: &str) -> String {
    format!("Φmock:{dependency}")
}

pub fn build_setup_line(member: &str, return_hint: &str) -> String {
    format!("Φsetup:{member} → {return_hint}")
}

pub fn build_verify_line(member: &str, times: Option<usize>) -> String {
    match times {
        Some(times) => format!("Φverify:{member} [times={times}]"),
        None => format!("Φverify:{member}"),
    }
}

pub fn build_fixture_line(method_name: &str) -> String {
    format!("Φfixture:{method_name}")
}
