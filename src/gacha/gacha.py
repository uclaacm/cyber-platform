from flask import Flask, request, render_template
from flask_sqlalchemy import SQLAlchemy
import os

app = Flask(__name__, static_folder=os.environ['STATIC_PATH'])
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ['DATABASE_URL']
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
db = SQLAlchemy(app)

from schema import *

@app.route('/')
@app.route('/rewards')
def gatcha():
    return render_template('rewards.html')

@app.route('/redeem', methods=['POST'])
def redeem():
    reward_type = request.form.get("type")
    if reward_type is None:
        return "Error"

    return "hey"