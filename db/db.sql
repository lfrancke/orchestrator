CREATE TABLE cluster_scoped
(
  api_group     TEXT,
  resource_type TEXT,
  resource_name TEXT,
  json          TEXT,
  PRIMARY KEY (api_group, resource_type, resource_name)
);

CREATE TABLE namespace_scoped
(
  api_group     TEXT,
  resource_type TEXT,
  namespace     TEXT,
  resource_name TEXT,
  json          TEXT,
  PRIMARY KEY (api_group, resource_type, namespace, resource_name)
);
