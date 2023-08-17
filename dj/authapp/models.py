from django.db import models

# Create your models here.
from django.db.models import Q
from django.contrib.auth.models import AbstractUser


class DateTimeBase(models.Model):
    created_on = models.DateTimeField(auto_now_add=True)
    updated_on = models.DateTimeField(auto_now=True)

    class Meta:
        abstract = True


class CustomUser(DateTimeBase):
    name = models.CharField(max_length=127, null=True)
    phone = models.CharField(max_length=20, null=True, unique=True)
    email = models.CharField(max_length=50, null=True, unique=True)
    active = (models.BooleanField(default=True),)
    last_login = (models.DateTimeField(null=True),)

    class Meta:
        db_table = "authapp_user"
        constraints = [
            models.CheckConstraint(
                check=Q(phone__isnull=False) | Q(email__isnull=False),
                name="phone_and_email_not_both_null_user",
            )
        ]


class UserToken(DateTimeBase):
    token = models.CharField(max_length=127)
    active = models.BooleanField(default=True)
    device_number = models.CharField(max_length=255, null=True)
    user = models.ForeignKey(CustomUser, on_delete=models.PROTECT)

    class Meta:
        db_table = "authapp_user_token"


class UserOtp(models.Model):
    email = models.CharField(max_length=50, null=True)
    phone = models.CharField(max_length=20, null=True)
    otp_bucket = models.JSONField()

    class Meta:
        db_table = "authapp_user_otp"
        constraints = [
            models.CheckConstraint(
                check=Q(phone__isnull=False) | Q(email__isnull=False),
                name="phone_and_email_not_both_null_otp",
            )
        ]
