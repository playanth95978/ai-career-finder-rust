import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { HttpResponse } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { IOfferPositioning } from '../offer-positioning.model';
import { OfferPositioningService } from '../service/offer-positioning.service';

import { OfferPositioningFormService } from './offer-positioning-form.service';
import { OfferPositioningUpdate } from './offer-positioning-update';

describe('OfferPositioning Management Update Component', () => {
  let comp: OfferPositioningUpdate;
  let fixture: ComponentFixture<OfferPositioningUpdate>;
  let activatedRoute: ActivatedRoute;
  let offerPositioningFormService: OfferPositioningFormService;
  let offerPositioningService: OfferPositioningService;
  let jobOfferService: JobOfferService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(OfferPositioningUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    offerPositioningFormService = TestBed.inject(OfferPositioningFormService);
    offerPositioningService = TestBed.inject(OfferPositioningService);
    jobOfferService = TestBed.inject(JobOfferService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should call JobOffer query and add missing value', () => {
      const offerPositioning: IOfferPositioning = { id: 9189 };
      const jobOffer: IJobOffer = { id: 9246 };
      offerPositioning.jobOffer = jobOffer;

      const jobOfferCollection: IJobOffer[] = [{ id: 9246 }];
      vitest.spyOn(jobOfferService, 'query').mockReturnValue(of(new HttpResponse({ body: jobOfferCollection })));
      const additionalJobOffers = [jobOffer];
      const expectedCollection: IJobOffer[] = [...additionalJobOffers, ...jobOfferCollection];
      vitest.spyOn(jobOfferService, 'addJobOfferToCollectionIfMissing').mockReturnValue(expectedCollection);

      activatedRoute.data = of({ offerPositioning });
      comp.ngOnInit();

      expect(jobOfferService.query).toHaveBeenCalled();
      expect(jobOfferService.addJobOfferToCollectionIfMissing).toHaveBeenCalledWith(
        jobOfferCollection,
        ...additionalJobOffers.map(i => expect.objectContaining(i) as typeof i),
      );
      expect(comp.jobOffersSharedCollection()).toEqual(expectedCollection);
    });

    it('should update editForm', () => {
      const offerPositioning: IOfferPositioning = { id: 9189 };
      const jobOffer: IJobOffer = { id: 9246 };
      offerPositioning.jobOffer = jobOffer;

      activatedRoute.data = of({ offerPositioning });
      comp.ngOnInit();

      expect(comp.jobOffersSharedCollection()).toContainEqual(jobOffer);
      expect(comp.offerPositioning).toEqual(offerPositioning);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IOfferPositioning>();
      const offerPositioning = { id: 28017 };
      vitest.spyOn(offerPositioningFormService, 'getOfferPositioning').mockReturnValue(offerPositioning);
      vitest.spyOn(offerPositioningService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ offerPositioning });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(offerPositioning);
      saveSubject.complete();

      // THEN
      expect(offerPositioningFormService.getOfferPositioning).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(offerPositioningService.update).toHaveBeenCalledWith(expect.objectContaining(offerPositioning));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IOfferPositioning>();
      const offerPositioning = { id: 28017 };
      vitest.spyOn(offerPositioningFormService, 'getOfferPositioning').mockReturnValue({ id: null });
      vitest.spyOn(offerPositioningService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ offerPositioning: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(offerPositioning);
      saveSubject.complete();

      // THEN
      expect(offerPositioningFormService.getOfferPositioning).toHaveBeenCalled();
      expect(offerPositioningService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IOfferPositioning>();
      const offerPositioning = { id: 28017 };
      vitest.spyOn(offerPositioningService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ offerPositioning });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(offerPositioningService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });

  describe('Compare relationships', () => {
    describe('compareJobOffer', () => {
      it('should forward to jobOfferService', () => {
        const entity = { id: 9246 };
        const entity2 = { id: 5985 };
        vitest.spyOn(jobOfferService, 'compareJobOffer');
        comp.compareJobOffer(entity, entity2);
        expect(jobOfferService.compareJobOffer).toHaveBeenCalledWith(entity, entity2);
      });
    });
  });
});
