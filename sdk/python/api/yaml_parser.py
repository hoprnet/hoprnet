import yaml
from yaml.resolver import BaseResolver


class PlainTextLoader(yaml.SafeLoader):
    pass

# Scalars always as strings
def construct_str(loader, node):
    return loader.construct_scalar(node)

# Sequences normal lists
def construct_seq(loader, node):
    return loader.construct_sequence(node, deep=True)

# Mappings normal dicts
def construct_mapping(loader, node):
    return loader.construct_mapping(node, deep=True)

# Catch-all for tags -> ignore the tag
def ignore_tag(loader, tag_suffix, node):
    if isinstance(node, yaml.ScalarNode):
        return loader.construct_scalar(node)
    elif isinstance(node, yaml.SequenceNode):
        return loader.construct_sequence(node, deep=True)
    elif isinstance(node, yaml.MappingNode):
        return loader.construct_mapping(node, deep=True)
    else:
        raise TypeError(f"Unsupported node type {type(node)}")


PlainTextLoader.yaml_implicit_resolvers = {}
PlainTextLoader.add_constructor(BaseResolver.DEFAULT_SCALAR_TAG, construct_str)
PlainTextLoader.add_constructor(BaseResolver.DEFAULT_SEQUENCE_TAG, construct_seq)
PlainTextLoader.add_constructor(BaseResolver.DEFAULT_MAPPING_TAG, construct_mapping)
PlainTextLoader.add_multi_constructor("!", ignore_tag)
