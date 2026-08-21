import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../job-offer.test-samples';

import { JobOfferFormService } from './job-offer-form.service';

describe('JobOffer Form Service', () => {
  let service: JobOfferFormService;

  beforeEach(() => {
    service = TestBed.inject(JobOfferFormService);
  });

  describe('Service methods', () => {
    describe('createJobOfferFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createJobOfferFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            title: expect.any(Object),
            company: expect.any(Object),
            location: expect.any(Object),
            country: expect.any(Object),
            remote: expect.any(Object),
            description: expect.any(Object),
            searchText: expect.any(Object),
            skills: expect.any(Object),
            metadata: expect.any(Object),
            rawPayload: expect.any(Object),
            contentHash: expect.any(Object),
            embeddingStatus: expect.any(Object),
            embeddingModel: expect.any(Object),
            reindexVersion: expect.any(Object),
            retryCount: expect.any(Object),
            indexingError: expect.any(Object),
            source: expect.any(Object),
            sourceId: expect.any(Object),
            applyUrl: expect.any(Object),
            salaryMin: expect.any(Object),
            salaryMax: expect.any(Object),
            salaryCurrency: expect.any(Object),
            contractType: expect.any(Object),
            experienceLevel: expect.any(Object),
            category: expect.any(Object),
            sourceCategory: expect.any(Object),
            publishedAt: expect.any(Object),
            createdAt: expect.any(Object),
            indexedAt: expect.any(Object),
            updatedAt: expect.any(Object),
            expiresAt: expect.any(Object),
            lastCheckedAt: expect.any(Object),
          }),
        );
      });

      it('passing IJobOffer should create a new form with FormGroup', () => {
        const formGroup = service.createJobOfferFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            title: expect.any(Object),
            company: expect.any(Object),
            location: expect.any(Object),
            country: expect.any(Object),
            remote: expect.any(Object),
            description: expect.any(Object),
            searchText: expect.any(Object),
            skills: expect.any(Object),
            metadata: expect.any(Object),
            rawPayload: expect.any(Object),
            contentHash: expect.any(Object),
            embeddingStatus: expect.any(Object),
            embeddingModel: expect.any(Object),
            reindexVersion: expect.any(Object),
            retryCount: expect.any(Object),
            indexingError: expect.any(Object),
            source: expect.any(Object),
            sourceId: expect.any(Object),
            applyUrl: expect.any(Object),
            salaryMin: expect.any(Object),
            salaryMax: expect.any(Object),
            salaryCurrency: expect.any(Object),
            contractType: expect.any(Object),
            experienceLevel: expect.any(Object),
            category: expect.any(Object),
            sourceCategory: expect.any(Object),
            publishedAt: expect.any(Object),
            createdAt: expect.any(Object),
            indexedAt: expect.any(Object),
            updatedAt: expect.any(Object),
            expiresAt: expect.any(Object),
            lastCheckedAt: expect.any(Object),
          }),
        );
      });
    });

    describe('getJobOffer', () => {
      it('should return NewJobOffer for default JobOffer initial value', () => {
        const formGroup = service.createJobOfferFormGroup(sampleWithNewData);

        const jobOffer = service.getJobOffer(formGroup);

        expect(jobOffer).toMatchObject(sampleWithNewData);
      });

      it('should return NewJobOffer for empty JobOffer initial value', () => {
        const formGroup = service.createJobOfferFormGroup();

        const jobOffer = service.getJobOffer(formGroup);

        expect(jobOffer).toMatchObject({});
      });

      it('should return IJobOffer', () => {
        const formGroup = service.createJobOfferFormGroup(sampleWithRequiredData);

        const jobOffer = service.getJobOffer(formGroup);

        expect(jobOffer).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IJobOffer should not enable id FormControl', () => {
        const formGroup = service.createJobOfferFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewJobOffer should disable id FormControl', () => {
        const formGroup = service.createJobOfferFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
