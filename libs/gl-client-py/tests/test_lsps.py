from glclient.lsps import AsDataClassDescriptor, EnhancedJSONEncoder, NoParams
import json

from dataclasses import dataclass

@dataclass
class Nested:
    value : str

@dataclass
class Nester:
    nested_field : Nested = AsDataClassDescriptor(cls=Nested)


def test_nested_serialization():
    nester = Nester(nested_field=Nested(value=0))
    json_str = json.dumps(nester, cls=EnhancedJSONEncoder)

    assert json_str == """{"nested_field": {"value": 0}}"""

def test_nested_deserialization():

    nested_dict = {
        "nested_field" : {"value" : 0}
    }
    result = Nester(**nested_dict)

    assert isinstance(result.nested_field, Nested)
    assert result.nested_field.value == 0

def test_serialize_no_params():
    no_params_1 = NoParams
    no_params_2 = NoParams()

    assert json.dumps(no_params_1, cls=EnhancedJSONEncoder) == "{}"
    assert json.dumps(no_params_2, cls=EnhancedJSONEncoder) == "{}"

def test_deserialize_no_params():
    json_str = "{}"

    # Should not raise
    # this behavior should be the same as for a dataclass

    NoParams(**json.loads(json_str))
