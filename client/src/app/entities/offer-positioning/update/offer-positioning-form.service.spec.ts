import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../offer-positioning.test-samples';

import { OfferPositioningFormService } from './offer-positioning-form.service';

describe('OfferPositioning Form Service', () => {
  let service: OfferPositioningFormService;

  beforeEach(() => {
    service = TestBed.inject(OfferPositioningFormService);
  });

  describe('Service methods', () => {
    describe('createOfferPositioningFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createOfferPositioningFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            result: expect.any(Object),
            createdAt: expect.any(Object),
            jobOffer: expect.any(Object),
          }),
        );
      });

      it('passing IOfferPositioning should create a new form with FormGroup', () => {
        const formGroup = service.createOfferPositioningFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            result: expect.any(Object),
            createdAt: expect.any(Object),
            jobOffer: expect.any(Object),
          }),
        );
      });
    });

    describe('getOfferPositioning', () => {
      it('should return NewOfferPositioning for default OfferPositioning initial value', () => {
        const formGroup = service.createOfferPositioningFormGroup(sampleWithNewData);

        const offerPositioning = service.getOfferPositioning(formGroup);

        expect(offerPositioning).toMatchObject(sampleWithNewData);
      });

      it('should return NewOfferPositioning for empty OfferPositioning initial value', () => {
        const formGroup = service.createOfferPositioningFormGroup();

        const offerPositioning = service.getOfferPositioning(formGroup);

        expect(offerPositioning).toMatchObject({});
      });

      it('should return IOfferPositioning', () => {
        const formGroup = service.createOfferPositioningFormGroup(sampleWithRequiredData);

        const offerPositioning = service.getOfferPositioning(formGroup);

        expect(offerPositioning).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IOfferPositioning should not enable id FormControl', () => {
        const formGroup = service.createOfferPositioningFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewOfferPositioning should disable id FormControl', () => {
        const formGroup = service.createOfferPositioningFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
