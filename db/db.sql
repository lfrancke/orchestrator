CREATE TABLE cluster_scoped (
  api_group TEXT,
  kind TEXT,
  resource_name TEXT,
  json TEXT,
  PRIMARY KEY (api_group, kind, resource_name)
);

CREATE TABLE namespace_scoped (
  api_group TEXT,
  kind TEXT,
  namespace TEXT,
  resource_name TEXT,
  json TEXT,
  PRIMARY KEY (api_group, kind, namespace, resource_name)
);
