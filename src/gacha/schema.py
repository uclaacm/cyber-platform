from datetime import datetime
from gacha import db


class Team(db.Model):
  __tablename__ = 'team'

  id = db.Column(db.Integer, primary_key=True)
  name = db.Column(db.String(), unique=True, nullable=False)
  discord = db.Column(db.String(), unique=True, nullable=False)
  hash = db.Column(db.String(), nullable=False)
  solves = db.Column(db.Integer, default=0)
  score = db.Column(db.Integer, default=0)
  redeemed_score = db.Column(db.Integer, default=0)
  premium_tickets = db.Column(db.Integer, default=0)
  isAdmin = db.Column(db.Boolean, default=False)
  submit = db.Column(db.DateTime, default=datetime.utcnow)


  def __init__(self, name, discord, hash):
    self.name = name
    self.discord = discord
    self.hash = hash

  def __repr__(self):
    return f"<Team {self.name}>"


class Prize(db.Model):
  __tablename__ = 'prize'

  id = db.Column(db.String(), nullable=False, primary_key=True)
  team = db.Column(db.Integer, db.ForeignKey('team.id'), nullable=False)
  prize = db.Column(db.String(), nullable=False)

  def __init__(self, team, prize):
    self.id = team+prize
    self.team = team
    self.prize = prize

  def __repr__(self):
    return f"<Prize {self.name}>"

class Raffle(db.Model):
  __tablename__ = 'raffle'

  team = db.Column(db.Integer, db.ForeignKey('team.id'), nullable=False)
  id = db.Column(db.Integer, primary_key=True)

  def __init__(self, team):
    self.team = team

  def __repr__(self):
    return f"<Raffle {self.name}>"
