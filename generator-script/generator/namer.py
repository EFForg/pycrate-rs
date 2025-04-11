import unittest
import inflect
import re


def ignore_token(token: str) -> bool:
    return len(token) == 0 or not token.isalnum()


inflect_engine = inflect.engine()
def tokenize(input: str) -> list[str]:
    """Splits an input string into a list of tokens, splitting not only on
    whitespace and punctuation, but also on camel-cased word boundaries (e.g.
    "FooBar" -> ["Foo", "Bar"])
    """
    tokens_re = re.compile(r"([A-Z][a-z]*|0x[\d]+|[\d]+|[^A-Za-z\d])")
    acronym = None
    tokens = []
    for token in re.split(tokens_re, input):
        if ignore_token(token):
            if token == '+':
                tokens.append('plus')
            if len(token) and acronym is not None:
                tokens.append(acronym)
                acronym = None
        elif len(token) == 1 and token.isupper():
            if acronym is None:
                acronym = token
            else:
                acronym += token
        else:
            if acronym is not None:
                tokens.append(acronym)
                acronym = None
            is_number = token.startswith('0x') or token.isnumeric()
            if is_number and len(tokens) == 0:
                number_word = inflect_engine.number_to_words(token)
                for word in tokenize(number_word):
                    tokens.append(word)
                continue
            tokens.append(token)
    if acronym is not None:
        tokens.append(acronym)
    return tokens


class Name:
    """Container for any string we'd like to conver to snake or camel case"""

    def __init__(self, raw: str):
        # special case this since hypens are annoying
        if raw == '-':
            self.words = ['minus']
        else:
            self.words = tokenize(raw)

    def cc(self):
        result = ''
        for word in self.words:
            if word[0].islower():
                result += f'{word[0].upper()}{word[1:]}'
            else:
                result += word
        return self._fix_reserved_words(result)

    def sc(self):
        result = '_'.join([word.lower() for word in self.words])
        return self._fix_reserved_words(result)

    def _fix_reserved_words(self, input: str) -> str:
        reserved = [
            'type',
        ]
        if input in reserved:
            return input[:-1]
        else:
            return input


class TestNamer(unittest.TestCase):
    def test_acronyms(self):
        tai_list = Name('TAIList')
        assert tai_list.cc() == 'TAIList'
        assert tai_list.sc() == 'tai_list'

        ue_rad = Name('UERadioCapIDDelInd')
        assert ue_rad.cc() == 'UERadioCapIDDelInd'
        assert ue_rad.sc() == 'ue_radio_cap_id_del_ind'

    def test_spaces(self):
        sms = Name('SMS services not available in this PLMN')
        assert sms.cc() == 'SMSServicesNotAvailableInThisPLMN'
        assert sms.sc() == 'sms_services_not_available_in_this_plmn'

    def test_punctuation(self):
        hypenated = Name('Non-EPS authentication unacceptable')
        assert hypenated.cc() == 'NonEPSAuthenticationUnacceptable'
        assert hypenated.sc() == 'non_eps_authentication_unacceptable'

        slashed = Name('TCP/IP')
        assert slashed.cc() == 'TCPIP'
        assert slashed.sc() == 'tcp_ip'

    def test_leading_numbers(self):
        bitrate = Name('200 kbps')
        assert bitrate.cc() == 'TwoHundredKbps'
        assert bitrate.sc() == 'two_hundred_kbps'

        # ugh why
        hex = Name('0x0000 (No Compression)')
        assert hex.cc() == 'ZeroNoCompression'
        assert hex.sc() == 'zero_no_compression'

        hex2 = Name('P0x0102')
        assert hex2.cc() == 'P0x0102'
        assert hex2.sc() == 'p_0x0102'

        hex3 = Name('0x0006 (TCP/IP)')
        assert hex3.cc() == 'SixTCPIP'
        assert hex3.sc() == 'six_tcp_ip'

    def test_reserved_word(self):
        type = Name('type')
        assert type.cc() == 'Type'
        assert type.sc() == 'typ'

    def test_plus_minus(self):
        plus = Name('+')
        assert plus.cc() == 'Plus'
        assert plus.sc() == 'plus'

        plus_two = Name('+Two')
        assert plus_two.cc() == 'PlusTwo'
        assert plus_two.sc() == 'plus_two'

        minus = Name('-')
        assert minus.cc() == 'Minus'
        assert minus.sc() == 'minus'



if __name__ == "__main__":
    unittest.main()
