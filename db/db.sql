CREATE TABLE cluster_scoped (
  api_group TEXT,
  resource_type_name TEXT,
  resource_name TEXT,
  json TEXT,
  PRIMARY KEY (api_group, resource_type_name, resource_name)
);

CREATE TABLE namespace_scoped (
  api_group TEXT,
  resource_type_name TEXT,
  namespace TEXT,
  resource_name TEXT,
  json TEXT,
  PRIMARY KEY (api_group, resource_type_name, namespace, resource_name)
);
